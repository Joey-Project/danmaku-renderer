use std::fmt::Write as _;
use std::time::Duration;

use crate::RenderStyle;

pub(crate) const WIDTH: f64 = 1920.0;
pub(crate) const HEIGHT: f64 = 1080.0;
pub(crate) const STYLE_FONT_SIZE: f64 = 42.0;

pub(crate) fn header(style: RenderStyle) -> String {
    let alpha = alpha(style);
    let back_alpha = if style.opacity == 1.0 { 0x64 } else { alpha };
    let font_size = (STYLE_FONT_SIZE * style.font_scale).round().max(1.0);
    format!(
        "[Script Info]\n\
         ScriptType: v4.00+\n\
         WrapStyle: 2\n\
         ScaledBorderAndShadow: yes\n\
         PlayResX: {WIDTH}\n\
         PlayResY: {HEIGHT}\n\n\
         [V4+ Styles]\n\
         Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n\
         Style: Danmaku,Arial,{font_size},&H{alpha:02X}FFFFFF,&H{alpha:02X}0000FF,&H{alpha:02X}000000,&H{back_alpha:02X}000000,0,0,0,0,100,100,0,0,1,2,0,7,20,20,20,1\n\n\
         [Events]\n\
         Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n"
    )
}

fn alpha(style: RenderStyle) -> u8 {
    ((1.0 - style.opacity) * 255.0).round() as u8
}

pub(crate) fn region_height(style: RenderStyle) -> f64 {
    (HEIGHT * style.display_area).floor().max(1.0)
}

pub(crate) fn write_dialogue(
    output: &mut String,
    time: std::ops::Range<u64>,
    positioning: &str,
    font_size: f64,
    color: u32,
    text: &str,
    style: RenderStyle,
) {
    let start = timestamp(time.start);
    let end = timestamp(time.end);
    let color = ass_color(color);
    let _ = write!(
        output,
        "Dialogue: 0,{start},{end},Danmaku,,0,0,0,,{{{positioning}\\fs{font_size:.0}\\c{color}",
    );
    if style.opacity != 1.0 {
        let alpha = alpha(style);
        let _ = write!(output, "\\alpha&H{alpha:02X}&");
    }
    if style.display_area != 1.0 {
        let height = region_height(style);
        let _ = write!(output, "\\clip(0,0,{WIDTH},{height})");
    }
    let _ = writeln!(output, "}}{text}");
}

pub(crate) fn centiseconds(seconds: f64) -> u64 {
    let seconds = seconds.clamp(0.0, 3_599_999.0);
    (Duration::from_secs_f64(seconds + 0.005).as_millis() / 10) as u64
}

fn timestamp(centiseconds: u64) -> String {
    let total_seconds = centiseconds / 100;
    let centisecond = centiseconds % 100;
    let second = total_seconds % 60;
    let minute = (total_seconds / 60) % 60;
    let hour = total_seconds / 3600;
    format!("{hour}:{minute:02}:{second:02}.{centisecond:02}")
}

pub(crate) fn ass_color(color: u32) -> String {
    let red = (color >> 16) & 0xFF;
    let green = (color >> 8) & 0xFF;
    let blue = color & 0xFF;
    format!("&H{blue:02X}{green:02X}{red:02X}&")
}

pub(crate) fn escape_text(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut output = String::with_capacity(normalized.len());
    for character in normalized.chars() {
        match character {
            '\n' => output.push_str("\\N"),
            '\\' => output.push_str(r"\\"),
            '{' => output.push_str("\\{"),
            '}' => output.push_str("\\}"),
            character if character.is_control() => {}
            character => output.push(character),
        }
    }
    output
}
