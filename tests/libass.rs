use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use danmaku_renderer::{RenderOptions, render_bilibili_xml};

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("libass-probe"));
        let _ = fs::remove_dir(&self.0);
    }
}

#[test]
#[ignore = "requires a C compiler, pkg-config, libass and a CJK font"]
fn compact_libass_paint_stays_in_region_and_keeps_fixed_comments_apart() {
    let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = std::env::temp_dir().join(format!("danmaku-libass-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let scratch = Scratch(dir);
    let probe = scratch.0.join("libass-probe");
    let flags = Command::new("pkg-config")
        .args(["--cflags", "--libs", "libass"])
        .output()
        .unwrap();
    assert!(flags.status.success(), "{:?}", flags.stderr);
    let compiled = Command::new("cc")
        .args(["-Wall", "-Wextra", "-Werror"])
        .arg(cwd.join("tests/support/libass_probe.c"))
        .args(
            std::str::from_utf8(&flags.stdout)
                .unwrap()
                .split_whitespace(),
        )
        .arg("-o")
        .arg(&probe)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );

    let xml = r#"<i>
        <d p="0,5,16,16711680">上沿红色</d>
        <d p="0,4,16,65280">下沿绿色</d>
        <d p="0.5,1,16,16777215">非常长的滚动弹幕 sample</d>
        <d p="1,6,20,16777215">反向弹幕</d></i>"#;
    for (area, height) in [(0.25, "270"), (0.5, "540"), (0.75, "810"), (1.0, "1080")] {
        let mut options = RenderOptions::default();
        options.style.display_area = area;
        let ass = render_bilibili_xml(xml, &options).unwrap();
        let mut child = Command::new(&probe)
            .arg(height)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(ass.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "area {area}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
