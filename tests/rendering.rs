use danmaku_renderer::{
    RenderError, RenderMode, RenderOptions, StylePreset, render_bilibili_xml,
    render_legacy_bilibili_xml,
};

fn events(ass: &str) -> Vec<&str> {
    ass.lines()
        .filter(|line| line.starts_with("Dialogue: "))
        .collect()
}

#[test]
fn naive_retains_order_entities_and_original_placement() {
    let xml = r#"<i><d p="8.25,4,25,16711680">bottom</d>
        <d p="0,7,25,0">advanced</d><d p="1,5,36,65280">top &amp; fixed</d>
        <d p="1,6,25,0">&#x5f39;&#24149;</d><d p="1,1,25,0">regular</d></i>"#;
    let options = RenderOptions::for_mode(RenderMode::Naive);
    let ass = render_bilibili_xml(xml, &options).unwrap();
    assert_eq!(ass, render_legacy_bilibili_xml(xml));
    let events = events(&ass);
    assert_eq!(events.len(), 4);
    assert!(events[0].contains(
        r"0:00:08.25,0:00:12.25,Danmaku,,0,0,0,,{\an2\pos(960,1040)\fs25\c&H0000FF&}bottom"
    ));
    assert!(events[1].ends_with(r"{\an8\pos(960,144)\fs36\c&H00FF00&}top & fixed"));
    assert!(events[2].ends_with(r"{\move(-50,196,1970,196)\fs25\c&H000000&}弹幕"));
    assert!(events[3].ends_with(r"{\move(2046,248,-126,248)\fs25\c&H000000&}regular"));
}

#[test]
fn strict_parsing_and_legacy_truncation_have_independent_contracts() {
    let truncated = r#"<i><d p="1,1,25,0">complete</d><d p="2,1,25,0">partial"#;
    assert!(matches!(
        render_bilibili_xml(truncated, &RenderOptions::default()),
        Err(RenderError::InvalidXml(_))
    ));
    let legacy = render_legacy_bilibili_xml(truncated);
    assert_eq!(events(&legacy).len(), 1);
    assert!(legacy.contains("complete"));
    assert!(matches!(
        render_bilibili_xml("<unrelated/>", &RenderOptions::default()),
        Err(RenderError::UnexpectedRoot)
    ));
}

#[test]
fn compact_dedup_is_ordered_by_time_and_separates_motion_types() {
    let xml = r#"<i>
        <d p="6,1,25,0">重复</d><d p="1,1,25,16711680">重复</d>
        <d p="1.2,2,25,0">重复</d><d p="1.5,5,25,0">重复</d>
        <d p="2.4,1,25,0">重复</d></i>"#;
    let options = RenderOptions::default();
    let ass = render_bilibili_xml(xml, &options).unwrap();
    let events = events(&ass);
    assert_eq!(events.len(), 3);
    assert!(events[0].contains(r"0:00:01.00,0:00:09.00"));
    assert!(events[0].ends_with(r"\fs75\c&H0000FF&\alpha&H4D&\clip(0,0,1920,540)}重复{\fs45} (3)"));
    assert!(events[1].contains(r"\an8\pos("));
    assert!(events[2].contains(r"0:00:06.00,0:00:14.00"));
}

#[test]
fn compact_quarter_screen_rejects_a_scroll_that_would_catch_a_shorter_one() {
    let xml = r#"<i>
        <d p="0,1,25,0">lead</d>
        <d p="1,1,25,0">弹幕弹幕弹幕弹幕弹幕弹幕弹幕弹幕弹幕弹幕弹幕弹幕</d>
        <d p="8,1,25,0">next</d></i>"#;
    let mut options = RenderOptions::default();
    options.style.display_area = 0.25;
    options.style.font_scale = 4.0;
    let ass = render_bilibili_xml(xml, &options).unwrap();
    assert_eq!(events(&ass).len(), 2);
    assert!(events(&ass)[0].ends_with("lead"));
    assert!(events(&ass)[1].ends_with("next"));
}

#[test]
fn compact_reuses_scroll_space_when_comments_keep_their_separation() {
    let xml = r#"<i>
        <d p="0,1,25,0">one</d><d p="1.5,1,25,0">two</d>
        <d p="2,6,25,0">reverse</d><d p="2,5,25,0">fixed</d></i>"#;
    let mut options = RenderOptions::default();
    options.style.display_area = 0.25;
    options.style.font_scale = 4.0;
    let ass = render_bilibili_xml(xml, &options).unwrap();
    assert_eq!(events(&ass).len(), 2);
    assert!(events(&ass)[0].ends_with("one"));
    assert!(events(&ass)[1].ends_with("two"));
}

#[test]
fn dense_input_obeys_display_capacity_and_does_not_move_timestamps() {
    use std::fmt::Write as _;
    let mut xml = String::from("<i>");
    for idx in 0..50_000 {
        write!(xml, r#"<d p="100,5,25,16777215">comment {idx}</d>"#).unwrap();
    }
    xml.push_str("</i>");

    let mut last = 0;
    for (fraction, extent) in [(0.25, 270), (0.50, 540), (0.75, 810), (1.0, 1080)] {
        let mut options = RenderOptions::default();
        options.style.display_area = fraction;
        let ass = render_bilibili_xml(&xml, &options).unwrap();
        let events = events(&ass);
        assert!(events.len() > last);
        last = events.len();
        for event in events {
            assert!(event.contains("0:01:40.00,0:01:44.00"));
            if fraction < 1.0 {
                assert!(event.contains(&format!(r"\clip(0,0,1920,{extent})")));
            }
        }
    }
}

#[test]
fn presets_are_independent_of_placement() {
    let xml = r#"<i><d p="1,1,25,0">same</d><d p="1.2,1,25,0">same</d></i>"#;
    let mut options = RenderOptions::for_mode(RenderMode::Naive);
    options.style = StylePreset::Tv.into();
    let ass = render_bilibili_xml(xml, &options).unwrap();
    assert_eq!(events(&ass).len(), 2);
    assert!(ass.contains(r"\fs75"));
    assert!(ass.contains(r"\alpha&H4D&\clip(0,0,1920,540)"));
    options.mode = RenderMode::Compact;
    options.style = StylePreset::Original.into();
    let ass = render_bilibili_xml(xml, &options).unwrap();
    assert_eq!(events(&ass).len(), 1);
    assert!(ass.contains(r"\fs25"));
    assert!(ass.contains(r"\clip(0,0,1920,1080)"));
}

#[test]
fn compact_keeps_literal_ass_controls_in_the_comment_text() {
    let xml = r#"<i><d p="1,5,25,0">literal \N and {\pos(0,900)}</d></i>"#;
    let ass = render_bilibili_xml(xml, &RenderOptions::default()).unwrap();
    let events = events(&ass);
    assert_eq!(events.len(), 1);
    assert!(events[0].ends_with("literal ＼N and ｛＼pos(0,900)｝"));
}
