// The original placement, durations and width estimate come from BBDown commit 7c3e54d.
use crate::{Comment, Motion, RenderStyle, ass};

const TOP_MARGIN: f64 = 40.0;
const LINE_HEIGHT: f64 = 52.0;
const SCROLL_LANES: usize = 15;
const FIXED_LANES: usize = 8;

pub(crate) fn render(comments: &[Comment], style: RenderStyle, escape: ass::Escape) -> String {
    let mut output = ass::header(style);
    let height = ass::region_height(style);
    let margin = TOP_MARGIN.min((height / 8.0).floor());
    let line_height = if style.font_scale == 1.0 {
        LINE_HEIGHT
    } else {
        comments
            .iter()
            .filter(|comment| comment.motion().is_some())
            .map(|comment| {
                comment.font_size(style) * 1.25 * comment.text.lines().count() as f64 + 4.0
            })
            .fold(1.0, f64::max)
            .ceil()
    };
    let lane_capacity = (((height - 2.0 * margin) / line_height).floor() as usize).max(1);

    for (index, comment) in comments.iter().enumerate() {
        let Some(motion) = comment.motion() else {
            continue;
        };
        let text = ass::escape_text(&comment.text, escape);
        if text.is_empty() {
            continue;
        }
        let start = ass::centiseconds(comment.start_seconds);
        let end = ass::centiseconds(comment.start_seconds + motion.duration());
        let font_size = comment.font_size(style);
        let center = ass::WIDTH / 2.0;
        let positioning = match motion {
            Motion::Bottom => {
                let y =
                    height - margin - (index % FIXED_LANES.min(lane_capacity)) as f64 * line_height;
                format!(r"\an2\pos({center:.0},{y:.0})")
            }
            Motion::Top => {
                let y = margin + (index % FIXED_LANES.min(lane_capacity)) as f64 * line_height;
                format!(r"\an8\pos({center:.0},{y:.0})")
            }
            Motion::ScrollLeft | Motion::ScrollRight => {
                let y = margin + (index % SCROLL_LANES.min(lane_capacity)) as f64 * line_height;
                let width =
                    (comment.text.chars().count() as f64 * font_size * 0.72).max(font_size * 2.0);
                let (from, to) = match motion {
                    Motion::ScrollRight => (-width, ass::WIDTH + width),
                    _ => (ass::WIDTH + width, -width),
                };
                format!(r"\move({from:.0},{y:.0},{to:.0},{y:.0})")
            }
        };
        ass::write_dialogue(
            &mut output,
            start..end,
            &positioning,
            font_size,
            comment.color,
            &text,
            style,
        );
    }
    output
}
