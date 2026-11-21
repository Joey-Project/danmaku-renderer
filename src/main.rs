use std::error::Error;
use std::fs;
use std::io::{self, Read as _, Write as _};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use danmaku_renderer::{RenderMode, RenderOptions, StylePreset, render_bilibili_xml};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Bilibili XML file, or "-" for standard input.
    input: PathBuf,
    /// ASS output file. Defaults to standard output.
    #[arg(short, long, default_value = "-")]
    output: PathBuf,
    /// Rendering algorithm.
    #[arg(long, value_enum, default_value = "compact")]
    mode: Mode,
    /// Override the mode's default style preset.
    #[arg(long, value_enum)]
    preset: Option<Preset>,
    /// Multiply each comment's font size by this factor (0 < scale <= 10).
    #[arg(long)]
    font_scale: Option<f64>,
    /// Text opacity, from 0 to 1.
    #[arg(long)]
    opacity: Option<f64>,
    /// Vertical area as a fraction (e.g. 1/4, 1/2, 3/4, 1), or decimal.
    #[arg(long, value_parser = parse_area)]
    display_area: Option<f64>,
    /// Compact-mode deduplication window in milliseconds. Zero disables it.
    #[arg(long, default_value_t = 1500)]
    dedup_window_ms: u64,
}

#[derive(Clone, Copy, ValueEnum)]
enum Mode {
    Naive,
    Compact,
}

#[derive(Clone, Copy, ValueEnum)]
enum Preset {
    Original,
    Tv,
}

fn parse_area(value: &str) -> Result<f64, String> {
    let parsed = if let Some((numerator, denominator)) = value.split_once('/') {
        numerator
            .parse::<f64>()
            .and_then(|n| denominator.parse::<f64>().map(|d| n / d))
    } else {
        value.parse()
    };
    parsed
        .ok()
        .filter(|area| 0.0 < *area && *area <= 1.0)
        .ok_or_else(|| "expected a fraction or decimal in (0, 1]".to_owned())
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let mut options = RenderOptions::for_mode(match args.mode {
        Mode::Naive => RenderMode::Naive,
        Mode::Compact => RenderMode::Compact,
    });
    if let Some(preset) = args.preset {
        options.style = match preset {
            Preset::Original => StylePreset::Original,
            Preset::Tv => StylePreset::Tv,
        }
        .into();
    }
    if let Some(scale) = args.font_scale {
        options.style.font_scale = scale;
    }
    if let Some(opacity) = args.opacity {
        options.style.opacity = opacity;
    }
    if let Some(area) = args.display_area {
        options.style.display_area = area;
    }
    options.dedup_window = Duration::from_millis(args.dedup_window_ms);

    let xml = if args.input.as_os_str() == "-" {
        let mut input = String::new();
        io::stdin().lock().read_to_string(&mut input)?;
        input
    } else {
        fs::read_to_string(args.input)?
    };
    let ass = render_bilibili_xml(&xml, &options)?;
    if args.output.as_os_str() == "-" {
        io::stdout().lock().write_all(ass.as_bytes())?;
    } else {
        fs::write(args.output, ass)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    if let Err(error) = run(Args::parse()) {
        eprintln!("danmaku-renderer: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
