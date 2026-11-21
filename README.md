# danmaku-renderer

Convert Bilibili XML danmaku to Advanced SubStation Alpha (ASS). The Rust library is available with no CLI dependencies. The optional command uses the same renderer.

## Install and use the CLI

```sh
cargo install danmaku-renderer --locked --features cli
danmaku-renderer danmaku.xml -o danmaku.ass
danmaku-renderer danmaku.xml --mode naive --preset tv --display-area 3/4 -o danmaku.ass
```

Use `-` for standard input. Output defaults to standard output. Use `--help` for the font, opacity, display-area and deduplication settings.

| Mode | Default preset | Placement |
| --- | --- | --- |
| `compact` (default) | `tv` | Groups repeated scrolling comments and uses their trajectories and timing to look ahead. Skips a comment if no space is available. |
| `naive` | `original` | Uses BBDown's original input order and round-robin placement. |

`original` uses the 1920 × 1080 BBDown layout, source font sizes and full opacity. `tv` scales source font sizes by 3, uses 70% opacity and confines comments to the upper half of the screen. Either mode accepts either preset. Explicit settings override the selected preset. `--display-area` accepts `1/4`, `1/2`, `3/4`, `1`, or other fractions and decimals in `(0, 1]`.

`compact` groups identical scrolling text traveling in the same direction within 1.5 seconds of the first comment, appending a repeat count. Set `--dedup-window-ms 0` to disable grouping. Placement estimates text bounds because font selection happens in the player.

## Use from Rust

```toml
[dependencies]
danmaku-renderer = "0.1.0"
```

```rust
use danmaku_renderer::{RenderOptions, render_bilibili_xml};

let xml = r#"<i><d p="1.2,1,25,16777215">弹幕</d></i>"#;
let mut options = RenderOptions::default();
options.style.display_area = 0.75;
let ass = render_bilibili_xml(xml, &options)?;
# Ok::<(), danmaku_renderer::RenderError>(())
```

`RenderOptions::for_mode` chooses the stated defaults for each mode. `options.style = StylePreset::Tv.into()` selects the TV preset explicitly. The renderer takes UTF-8 XML with a Bilibili `<i>` root. Advanced modes 7 and 8 are skipped.

## Development and publication

```sh
cargo test --locked --no-default-features
cargo test --locked --all-features
cargo clippy --locked --all-features --all-targets -- -D warnings
cargo test --locked --test libass -- --ignored
```

The libass check uses a local C compiler, `pkg-config`, libass headers, and a CJK font to inspect rendered pixels.

The golden tests compare two [Bilibili fixtures](tests/fixtures/README.md) against snapshots from BBDown and Telegram. The library also exposes prefix rendering by Compact layout version so an application can verify previously generated ASS before replacing it.

On GitHub, the publish workflow accepts a version from `master`, verifies the matching `v<version>` tag and Cargo package, and publishes from the protected `crates-io` environment. Set `CARGO_REGISTRY_TOKEN` in that environment for the first release and select `bootstrap` when dispatching. Subsequent releases use Trusted Publishing for `Joey-Project/danmaku-renderer`, `release.yml`, and the `crates-io` environment. Remove the bootstrap credential after the initial publication.

Licensed under MIT. Original placement comes from [BBDown-rust](https://github.com/Joey-Project/BBDown-rust/commit/7c3e54d). Compact grouping and lookahead come from [Telegram-Video-Downloader](https://github.com/Joey-Project/Telegram-Video-Downloader/commit/9d0a5c3c82ffa957cbf33cbdf07dfba059e2cbdd).
