use roxmltree::Document;

use crate::{Comment, RenderError};

pub(crate) fn parse(xml: &str) -> Result<Vec<Comment>, RenderError> {
    let document = Document::parse(xml).map_err(RenderError::InvalidXml)?;
    let root = document.root_element();
    if !root.has_tag_name("i") {
        return Err(RenderError::UnexpectedRoot);
    }

    Ok(root
        .children()
        .filter(|node| node.has_tag_name("d"))
        .filter_map(|node| {
            let parameters = node.attribute("p")?;
            let text = node
                .children()
                .filter_map(|child| child.is_text().then(|| child.text()).flatten())
                .collect();
            Comment::parse(parameters, text)
        })
        .collect())
}

// Legacy BBDown XML scanner from 7c3e54d. Keep its best-effort recovery behavior.
pub(crate) fn parse_legacy(xml: &str) -> Vec<Comment> {
    let mut comments = Vec::new();
    let mut offset = 0;
    while let Some(tag_start_relative) = xml[offset..].find("<d") {
        let tag_start = offset + tag_start_relative;
        let name_tail = tag_start + 2;
        let Some(next) = xml[name_tail..].chars().next() else {
            break;
        };
        if !(next.is_ascii_whitespace() || matches!(next, '>' | '/')) {
            offset = name_tail;
            continue;
        }
        let Some(tag_end_relative) = xml[tag_start..].find('>') else {
            break;
        };
        let tag_end = tag_start + tag_end_relative;
        let content_start = tag_end + 1;
        let Some(close_relative) = xml[content_start..].find("</d>") else {
            break;
        };
        let content_end = content_start + close_relative;
        offset = content_end + "</d>".len();

        let Some(parameters) = attribute_value(&xml[tag_start..tag_end], "p") else {
            continue;
        };
        if let Some(comment) =
            Comment::parse(&parameters, xml_unescape(&xml[content_start..content_end]))
        {
            comments.push(comment);
        }
    }
    comments
}

fn attribute_value(tag: &str, name: &str) -> Option<String> {
    let bytes = tag.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        while index < bytes.len() && !is_attribute_name_char(bytes[index]) {
            index += 1;
        }
        let name_start = index;
        while index < bytes.len() && is_attribute_name_char(bytes[index]) {
            index += 1;
        }
        if name_start == index {
            continue;
        }
        let attr_name = &tag[name_start..index];
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if bytes.get(index) != Some(&b'=') {
            continue;
        }
        index += 1;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        let quote = bytes.get(index).copied()?;
        if !matches!(quote, b'\'' | b'"') {
            continue;
        }
        index += 1;
        let value_start = index;
        while index < bytes.len() && bytes[index] != quote {
            index += 1;
        }
        if index >= bytes.len() {
            return None;
        }
        if attr_name == name {
            return Some(xml_unescape(&tag[value_start..index]));
        }
        index += 1;
    }
    None
}

const fn is_attribute_name_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':')
}

fn xml_unescape(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while let Some(ampersand_relative) = text[index..].find('&') {
        let ampersand = index + ampersand_relative;
        output.push_str(&text[index..ampersand]);
        let entity_start = ampersand + 1;
        let Some(semicolon_relative) = text[entity_start..].find(';') else {
            output.push_str(&text[ampersand..]);
            return output;
        };
        let semicolon = entity_start + semicolon_relative;
        let entity = &text[entity_start..semicolon];
        if let Some(decoded) = decode_xml_entity(entity) {
            output.push(decoded);
            index = semicolon + 1;
        } else {
            output.push('&');
            index = entity_start;
        }
    }
    output.push_str(&text[index..]);
    output
}

fn decode_xml_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        entity if entity.starts_with("#x") || entity.starts_with("#X") => {
            u32::from_str_radix(&entity[2..], 16)
                .ok()
                .and_then(char::from_u32)
        }
        entity if entity.starts_with('#') => {
            entity[1..].parse::<u32>().ok().and_then(char::from_u32)
        }
        _ => None,
    }
}
