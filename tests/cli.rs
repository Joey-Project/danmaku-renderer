#![cfg(feature = "cli")]

use std::io::Write as _;
use std::process::{Command, Output, Stdio};

const XML: &[u8] = br#"<i><d p="1,5,25,0">A comment</d></i>"#;

fn cli(args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_danmaku-renderer"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn cli_stdin_to_stdout_produces_the_same_ass_as_the_library() {
    let output = cli(
        &[
            "-",
            "--mode",
            "naive",
            "--preset",
            "tv",
            "--display-area",
            "3/4",
        ],
        XML,
    );
    assert!(output.status.success(), "{:?}", output.stderr);
    let mut options =
        danmaku_renderer::RenderOptions::for_mode(danmaku_renderer::RenderMode::Naive);
    options.style = danmaku_renderer::StylePreset::Tv.into();
    options.style.display_area = 0.75;
    let expected =
        danmaku_renderer::render_bilibili_xml(std::str::from_utf8(XML).unwrap(), &options).unwrap();
    assert_eq!(output.stdout, expected.as_bytes());
}

#[test]
fn cli_rejects_malformed_xml_and_invalid_options() {
    let malformed = cli(&["-"], b"<i><d");
    assert!(!malformed.status.success());
    assert!(malformed.stdout.is_empty());
    assert!(String::from_utf8_lossy(&malformed.stderr).contains("invalid danmaku XML"));

    let invalid = cli(&["-", "--display-area", "1/0"], b"");
    assert!(!invalid.status.success());
    assert!(invalid.stdout.is_empty());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("expected a fraction"));
}
