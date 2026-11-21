use std::io::Read as _;

use danmaku_renderer::{
    CompactLayoutVersion, RenderMode, RenderOptions, render_bilibili_xml, render_compact_prefix,
    render_legacy_bilibili_xml,
};
use flate2::read::GzDecoder;

struct Sample {
    xml: &'static [u8],
    ass: &'static [u8],
    deployed_ass: &'static [u8],
    count: usize,
}

const CHIP_EFFICIENCY: Sample = Sample {
    xml: include_bytes!("fixtures/chip-efficiency.xml.gz"),
    ass: include_bytes!("fixtures/chip-efficiency.naive.ass.gz"),
    deployed_ass: include_bytes!("fixtures/chip-efficiency.compact-v3.ass.gz"),
    count: 10_465,
};

const BLOOM_INTO_YOU_STAGE: Sample = Sample {
    xml: include_bytes!("fixtures/bloom-into-you-stage.xml.gz"),
    ass: include_bytes!("fixtures/bloom-into-you-stage.naive.ass.gz"),
    deployed_ass: include_bytes!("fixtures/bloom-into-you-stage.compact-v3.ass.gz"),
    count: 9_331,
};

fn gunzip(bytes: &[u8]) -> String {
    let mut content = String::new();
    GzDecoder::new(bytes).read_to_string(&mut content).unwrap();
    content
}

fn check(sample: Sample) {
    let xml = gunzip(sample.xml);
    let expected = gunzip(sample.ass);
    for actual in [
        render_bilibili_xml(&xml, &RenderOptions::for_mode(RenderMode::Naive)).unwrap(),
        render_legacy_bilibili_xml(&xml),
    ] {
        assert!(
            actual == expected,
            "BBDown snapshot mismatch, first differing line: {:?}, line counts: {} vs {}",
            actual
                .lines()
                .zip(expected.lines())
                .position(|(a, b)| a != b)
                .map(|line| line + 1),
            actual.lines().count(),
            expected.lines().count(),
        );
    }
    assert_eq!(
        expected
            .lines()
            .filter(|line| line.starts_with("Dialogue: "))
            .count(),
        sample.count
    );

    let compact_options = RenderOptions::default();
    let original = render_compact_prefix(
        &xml,
        &compact_options,
        sample.count,
        CompactLayoutVersion::V3,
    )
    .unwrap();
    let original_ass = gunzip(sample.deployed_ass);
    assert!(
        original.ass == original_ass,
        "Telegram v3 snapshot mismatch, first differing line: {:?}",
        original
            .ass
            .lines()
            .zip(original_ass.lines())
            .position(|(a, b)| a != b)
            .map(|line| line + 1),
    );
    let compact = render_bilibili_xml(&xml, &compact_options).unwrap();
    let events = compact
        .lines()
        .filter(|line| line.starts_with("Dialogue: "));
    let mut count = 0;
    for event in events {
        assert!(event.contains(r"\alpha&H4D&\clip(0,0,1920,540)"));
        count += 1;
    }
    assert!(
        (100..sample.count).contains(&count),
        "{count} compact events"
    );
}

#[test]
fn chip_efficiency_golden() {
    check(CHIP_EFFICIENCY);
}

#[test]
fn bloom_into_you_stage_golden() {
    check(BLOOM_INTO_YOU_STAGE);
}
