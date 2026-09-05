use descar_core::file::{FileSizeInfo, FileSizeReport, FormattedSize, FormattedSizePair, SizeSystem, SizeSystems};
use insta::assert_snapshot;

#[test]
fn snapshots_size_system_debug_representations() {
    let custom = SizeSystem::new("custom_sys", 100.0, ["u0", "u1", "u2", "u3", "u4", "u5"]).unwrap();
    let rendered = format!("si={:?}\niec={:?}\ncustom={:?}", SizeSystems::SI_SYSTEM, SizeSystems::IEC, custom);
    assert_snapshot!("size_system_debug_representations", rendered);
}

#[test]
fn snapshots_formatted_sizes() {
    let cases = [
        FormattedSize::new(0.0, "B"),
        FormattedSize::new(0.004, "B"),
        FormattedSize::new(0.999, "KB"),
        FormattedSize::new(1.0, "KiB"),
        FormattedSize::new(12.3456, "MB"),
        FormattedSize::new(999.994, "GB"),
        FormattedSize::new(1023.999, "TiB"),
        FormattedSize::new(1024.0, "PiB"),
    ];

    let rendered = cases.iter().map(|s| format!("{s:?} => {s}")).collect::<Vec<_>>().join("\n");

    assert_snapshot!("formatted_sizes", rendered);
}

#[test]
fn snapshots_formatted_size_pairs() {
    let cases = [
        (0, "zero"),
        (500, "sub_kilo"),
        (1_000, "exact_1k_decimal"),
        (1_024, "exact_1k_binary"),
        (1_500_000, "megabytes_range"),
        (1_073_741_824, "gigabyte_binary"),
        (1_000_000_000_000, "terabyte_decimal"),
        (1_125_899_906_842_624, "petabyte_binary"),
        (u64::MAX, "u64_max"),
    ];

    let rendered = cases
        .iter()
        .map(|(bytes, label)| {
            let info = FileSizeInfo::new(*bytes);
            let pair = FormattedSizePair::new(info.format(&SizeSystems::SI_SYSTEM), info.format(&SizeSystems::IEC));
            format!("[{label} - {bytes} bytes]\n{pair}")
        })
        .collect::<Vec<_>>()
        .join("\n---\n");

    assert_snapshot!("formatted_size_pairs", rendered);
}

#[test]
fn snapshots_file_size_reports() {
    let test_bytes = [0, 1, 999, 1_000, 1_023, 1_024, 1_048_576, 1_000_000_000, 1_099_511_627_776, u64::MAX];

    let rendered = test_bytes
        .iter()
        .map(|bytes| {
            let report = FileSizeReport::new(FileSizeInfo::new(*bytes), &SizeSystems::SI_SYSTEM, &SizeSystems::IEC);
            report.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n=========================================\n");

    assert_snapshot!("file_size_reports", rendered);
}
