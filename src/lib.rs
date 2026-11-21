//! Convert Bilibili XML danmaku to ASS subtitles.

mod ass;
mod compact;
mod naive;
mod xml;

use std::fmt;
use std::time::Duration;

pub use compact::{CompactLayoutVersion, MAX_XML_BYTES, RenderedAss};

/// The algorithm that places comments on the ASS canvas.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RenderMode {
    /// Preserve BBDown's input order and round-robin lane assignment.
    Naive,
    /// Suppress recent duplicates and assign space using comment trajectories.
    #[default]
    Compact,
}

/// Defaults for size, opacity and vertical display area.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StylePreset {
    /// The original BBDown style.
    Original,
    /// Larger, translucent text within the top half of the canvas.
    Tv,
}

/// Appearance and display region, independently configurable for each mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderStyle {
    /// Scale applied to each comment's font size. Valid range is (0, 10].
    pub font_scale: f64,
    /// Text and outline opacity. Valid range is [0, 1].
    pub opacity: f64,
    /// Fraction of the canvas height available, measured down from the top. Valid range is (0, 1].
    pub display_area: f64,
}

impl From<StylePreset> for RenderStyle {
    fn from(preset: StylePreset) -> Self {
        match preset {
            StylePreset::Original => Self {
                font_scale: 1.0,
                opacity: 1.0,
                display_area: 1.0,
            },
            StylePreset::Tv => Self {
                font_scale: 3.0,
                opacity: 0.7,
                display_area: 0.5,
            },
        }
    }
}

/// Options shared by the Rust API and CLI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderOptions {
    pub mode: RenderMode,
    pub style: RenderStyle,
    /// Compact groups identical scrolling text moving in the same direction within this
    /// window after the first comment. Millisecond precision, up to ten seconds. Zero
    /// disables grouping.
    pub dedup_window: Duration,
}

impl RenderOptions {
    /// Select a mode and its default preset.
    #[must_use]
    pub fn for_mode(mode: RenderMode) -> Self {
        let preset = match mode {
            RenderMode::Naive => StylePreset::Original,
            RenderMode::Compact => StylePreset::Tv,
        };
        Self {
            mode,
            style: preset.into(),
            dedup_window: Duration::from_millis(1500),
        }
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self::for_mode(RenderMode::default())
    }
}

/// An unusable XML document or an invalid display setting.
#[derive(Debug)]
#[non_exhaustive]
pub enum RenderError {
    InvalidXml(roxmltree::Error),
    UnexpectedRoot,
    InvalidFontScale,
    InvalidOpacity,
    InvalidDisplayArea,
    InvalidDedupWindow,
    Compact(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(error) => write!(formatter, "invalid danmaku XML: {error}"),
            Self::UnexpectedRoot => formatter.write_str("expected a Bilibili <i> root element"),
            Self::InvalidFontScale => formatter.write_str("font scale must be in (0, 10]"),
            Self::InvalidOpacity => formatter.write_str("opacity must be in [0, 1]"),
            Self::InvalidDisplayArea => formatter.write_str("display area must be in (0, 1]"),
            Self::InvalidDedupWindow => {
                formatter.write_str("dedup window must be at most 10000 ms")
            }
            Self::Compact(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidXml(error) => Some(error),
            _ => None,
        }
    }
}

/// Render a well-formed Bilibili XML document. Individual comments with unusable
/// timestamps, mode fields or text are skipped, as are advanced modes 7 and 8.
///
/// # Errors
///
/// Returns an error for malformed XML, an unexpected root, or an invalid style.
pub fn render_bilibili_xml(xml: &str, options: &RenderOptions) -> Result<String, RenderError> {
    match options.mode {
        RenderMode::Naive => {
            validate_style(options.style)?;
            let comments = xml::parse(xml)?;
            Ok(naive::render(&comments, options.style))
        }
        RenderMode::Compact => render_compact_ass(xml, options).map(|rendered| rendered.ass),
    }
}

/// Render a complete Compact ASS and return its accepted source comment count.
pub fn render_compact_ass(xml: &str, options: &RenderOptions) -> Result<RenderedAss, RenderError> {
    render_compact(xml, options, None, CompactLayoutVersion::V4)
}

/// Reproduce a Compact ASS from a prefix of accepted comments in XML storage order.
/// Select the layout version recorded when that sidecar was generated.
pub fn render_compact_prefix(
    xml: &str,
    options: &RenderOptions,
    source_comments: usize,
    version: CompactLayoutVersion,
) -> Result<RenderedAss, RenderError> {
    render_compact(xml, options, Some(source_comments), version)
}

fn render_compact(
    xml: &str,
    options: &RenderOptions,
    source_comments: Option<usize>,
    version: CompactLayoutVersion,
) -> Result<RenderedAss, RenderError> {
    validate_style(options.style)?;
    if options.dedup_window > Duration::from_secs(10) {
        return Err(RenderError::InvalidDedupWindow);
    }
    let settings = compact::RenderOptions {
        font_scale: options.style.font_scale,
        opacity: options.style.opacity,
        display_area: options.style.display_area,
        merge_window_ms: options.dedup_window.as_millis() as u64,
    };
    compact::render_ass_with_limit(xml, &settings, source_comments, version).map_err(|error| {
        match error.downcast::<RenderError>() {
            Ok(error) => error,
            Err(error) => RenderError::Compact(error.to_string()),
        }
    })
}

fn validate_style(style: RenderStyle) -> Result<(), RenderError> {
    if !(0.0 < style.font_scale && style.font_scale <= 10.0) {
        return Err(RenderError::InvalidFontScale);
    }
    if !(0.0..=1.0).contains(&style.opacity) {
        return Err(RenderError::InvalidOpacity);
    }
    if !(0.0 < style.display_area && style.display_area <= 1.0) {
        return Err(RenderError::InvalidDisplayArea);
    }

    Ok(())
}

/// Render with the original BBDown scanner and style, recovering complete
/// comments from some malformed or truncated XML inputs.
///
/// Use this entry point for compatibility with BBDown's legacy update path.
#[must_use]
pub fn render_legacy_bilibili_xml(xml: &str) -> String {
    naive::render(&xml::parse_legacy(xml), StylePreset::Original.into())
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum Motion {
    ScrollLeft,
    ScrollRight,
    Top,
    Bottom,
}

impl Motion {
    fn duration(self) -> f64 {
        match self {
            Self::Top | Self::Bottom => 4.0,
            Self::ScrollLeft | Self::ScrollRight => 8.0,
        }
    }
}

struct Comment {
    start_seconds: f64,
    mode: u8,
    font_size: f64,
    color: u32,
    text: String,
}

impl Comment {
    fn parse(parameters: &str, text: String) -> Option<Self> {
        let mut parts = parameters.split(',');
        let start_seconds = parts.next()?.parse::<f64>().ok()?;
        if !start_seconds.is_finite() || text.is_empty() {
            return None;
        }
        let mode = parts.next()?.parse::<u8>().ok()?;
        let font_size = parts
            .next()
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|size| size.is_finite())
            .map_or(ass::STYLE_FONT_SIZE, |size| size.clamp(12.0, 96.0));
        let color = parts
            .next()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0xFF_FF_FF)
            & 0xFF_FF_FF;
        Some(Self {
            start_seconds: start_seconds.max(0.0),
            mode,
            font_size,
            color,
            text,
        })
    }

    fn motion(&self) -> Option<Motion> {
        match self.mode {
            4 => Some(Motion::Bottom),
            5 => Some(Motion::Top),
            6 => Some(Motion::ScrollRight),
            7 | 8 => None,
            _ => Some(Motion::ScrollLeft),
        }
    }

    fn font_size(&self, style: RenderStyle) -> f64 {
        (self.font_size * style.font_scale).round().max(1.0)
    }
}
