#![allow(clippy::float_cmp, clippy::unreadable_literal)]

use descar_core::file::{FileSizeInfo, FileSizeReport, FormattedSize, FormattedSizePair, SizeSystem, SizeSystems};

// =========================================================================
// SizeSystem Unit, Corner & Edge Case Tests
// =========================================================================

#[test]
fn test_size_system_new_valid() {
    let prefixes = ["b", "k", "m", "g", "t", "p"];
    let sys = SizeSystem::new("custom", 10.0, prefixes).expect("valid base >= 1.0");
    assert_eq!(sys.name(), "custom");
    assert_eq!(sys.base(), 10.0);
    assert_eq!(sys.prefixes(), &prefixes);
}

#[test]
fn test_size_system_exact_expected_base_boundary() {
    // Base exactly at EXPECTED_BASE (1.0) is allowed
    let prefixes = ["0", "1", "2", "3", "4", "5"];
    let sys = SizeSystem::new("unit_base", SizeSystem::EXPECTED_BASE, prefixes);
    assert!(sys.is_ok());
    let sys = sys.unwrap();
    assert_eq!(sys.base(), 1.0);
}

#[test]
fn test_size_system_new_err_below_expected_base() {
    let prefixes = ["0", "1", "2", "3", "4", "5"];

    // Base just below 1.0
    let err_just_below = SizeSystem::new("invalid", 0.999_999_999_999_999, prefixes);
    assert_eq!(err_just_below, Err("base must be >= 1.0"));

    // Base zero
    let err_zero = SizeSystem::new("invalid", 0.0, prefixes);
    assert_eq!(err_zero, Err("base must be >= 1.0"));

    // Base negative
    let err_neg = SizeSystem::new("invalid", -1000.0, prefixes);
    assert_eq!(err_neg, Err("base must be >= 1.0"));

    // Base negative infinity
    let err_neg_inf = SizeSystem::new("invalid", f64::NEG_INFINITY, prefixes);
    assert_eq!(err_neg_inf, Err("base must be >= 1.0"));
}

#[test]
fn test_size_systems_constants() {
    assert_eq!(SizeSystem::PREFIX_COUNT, 6);
    assert_eq!(SizeSystem::EXPECTED_BASE, 1.0);

    // SI
    assert_eq!(SizeSystems::SI_SYSTEM.name(), "SI");
    assert_eq!(SizeSystems::SI_SYSTEM.base(), 1000.0);
    assert_eq!(SizeSystems::SI_SYSTEM.prefixes(), &["B", "KB", "MB", "GB", "TB", "PB"]);

    // IEC
    assert_eq!(SizeSystems::IEC.name(), "IEC");
    assert_eq!(SizeSystems::IEC.base(), 1024.0);
    assert_eq!(SizeSystems::IEC.prefixes(), &["B", "KiB", "MiB", "GiB", "TiB", "PiB"]);
}

#[test]
fn test_size_system_traits_clone_and_equality() {
    let sys1 = SizeSystems::SI_SYSTEM;
    let sys2 = sys1.clone();
    assert_eq!(sys1, sys2);

    let iec = SizeSystems::IEC;
    assert_ne!(sys1, iec);

    // Debug representation
    let debug_str = format!("{sys1:?}");
    assert!(debug_str.contains("SI"));
    assert!(debug_str.contains("1000"));
}

// =========================================================================
// FormattedSize Unit, Corner & Edge Case Tests
// =========================================================================

#[test]
fn test_formatted_size_new_and_fields() {
    let fs = FormattedSize::new(123.456, "MB");
    assert_eq!(fs.value, 123.456);
    assert_eq!(fs.suffix, "MB");
}

#[test]
fn test_formatted_size_display() {
    // 2 decimal places rounding check
    let fs = FormattedSize::new(12.345, "KB");
    assert_eq!(format!("{fs}"), "12.35 KB");

    let fs_zero = FormattedSize::new(0.0, "B");
    assert_eq!(format!("{fs_zero}"), "0.00 B");

    let fs_int = FormattedSize::new(5.0, "GB");
    assert_eq!(format!("{fs_int}"), "5.00 GB");
}

#[test]
fn test_formatted_size_copy_and_equality() {
    let fs1 = FormattedSize::new(1.0, "B");
    let fs2 = fs1; // Copy
    assert_eq!(fs1, fs2);
    assert_eq!(fs1.clone(), fs2);

    let fs3 = FormattedSize::new(1.0, "KiB");
    assert_ne!(fs1, fs3);

    let fs4 = FormattedSize::new(2.0, "B");
    assert_ne!(fs1, fs4);
}

// =========================================================================
// FormattedSizePair Unit, Corner & Edge Case Tests
// =========================================================================

#[test]
fn test_formatted_size_pair_new_and_display() {
    let si = FormattedSize::new(1.0, "KB");
    let iec = FormattedSize::new(1000.0 / 1024.0, "B");
    let pair = FormattedSizePair::new(si, iec);

    assert_eq!(pair.si_size, si);
    assert_eq!(pair.iec_size, iec);

    let display = format!("{pair}");
    // Checks that fields are formatted with 20 width left-aligned: "{si_str:<20} {iec_str:<20}"
    let si_str = si.to_string();
    let iec_str = iec.to_string();
    assert_eq!(display, format!("{si_str:<20} {iec_str:<20}"));
}

#[test]
fn test_formatted_size_pair_copy_and_equality() {
    let si = FormattedSize::new(500.0, "B");
    let iec = FormattedSize::new(500.0, "B");
    let pair1 = FormattedSizePair::new(si, iec);
    let pair2 = pair1; // Copy
    assert_eq!(pair1, pair2);
    assert_eq!(pair1.clone(), pair2);
}

// =========================================================================
// FileSizeInfo Unit, Corner & Edge Case Tests
// =========================================================================

#[test]
fn test_file_size_info_default_and_new() {
    let def = FileSizeInfo::default();
    assert_eq!(def.bytes, 0);

    let info = FileSizeInfo::new(1024);
    assert_eq!(info.bytes, 1024);
}

#[test]
fn test_file_size_info_copy_and_equality() {
    let a = FileSizeInfo::new(42);
    let b = a; // Copy
    assert_eq!(a, b);
    assert_eq!(a.clone(), b);

    let c = FileSizeInfo::new(43);
    assert_ne!(a, c);
}

#[test]
fn test_file_size_info_format_zero_bytes() {
    let info = FileSizeInfo::new(0);

    let formatted_si = info.format(&SizeSystems::SI_SYSTEM);
    assert_eq!(formatted_si.value, 0.0);
    assert_eq!(formatted_si.suffix, "B");

    let formatted_iec = info.format(&SizeSystems::IEC);
    assert_eq!(formatted_iec.value, 0.0);
    assert_eq!(formatted_iec.suffix, "B");
}

#[test]
fn test_file_size_info_format_exact_tier_boundaries() {
    let iec = &SizeSystems::IEC;

    // 1023 B -> remains B
    let b1023 = FileSizeInfo::new(1023).format(iec);
    assert_eq!(b1023.value, 1023.0);
    assert_eq!(b1023.suffix, "B");

    // 1024 B -> exactly 1.00 KiB
    let kib1 = FileSizeInfo::new(1024).format(iec);
    assert_eq!(kib1.value, 1.0);
    assert_eq!(kib1.suffix, "KiB");

    // 1024^2 - 1 -> just below 1 MiB
    let below_mib = FileSizeInfo::new(1024 * 1024 - 1).format(iec);
    assert_eq!(below_mib.suffix, "KiB");

    // 1024^2 -> exactly 1.00 MiB
    let mib1 = FileSizeInfo::new(1024 * 1024).format(iec);
    assert_eq!(mib1.value, 1.0);
    assert_eq!(mib1.suffix, "MiB");

    // 1024^3 -> 1.00 GiB
    let gib1 = FileSizeInfo::new(1024 * 1024 * 1024).format(iec);
    assert_eq!(gib1.value, 1.0);
    assert_eq!(gib1.suffix, "GiB");

    // 1024^4 -> 1.00 TiB
    let tib1 = FileSizeInfo::new(1024 * 1024 * 1024 * 1024).format(iec);
    assert_eq!(tib1.value, 1.0);
    assert_eq!(tib1.suffix, "TiB");

    // 1024^5 -> 1.00 PiB
    let pib1 = FileSizeInfo::new(1024 * 1024 * 1024 * 1024 * 1024).format(iec);
    assert_eq!(pib1.value, 1.0);
    assert_eq!(pib1.suffix, "PiB");
}

#[test]
fn test_file_size_info_format_exceeding_max_prefix_index() {
    let iec = &SizeSystems::IEC;

    // 1024^6 (Exabytes) - since MAX_PREFIX_INDEX is 5, it should stay at index 5 ("PiB")
    // and value should be >= 1024.0 PiB.
    let eib1 = 1024u64.pow(6 - 1) * 1024;
    let formatted = FileSizeInfo::new(eib1).format(iec);
    assert_eq!(formatted.suffix, "PiB");
    assert_eq!(formatted.value, 1024.0);

    // u64::MAX boundary
    let max_info = FileSizeInfo::new(u64::MAX);
    let max_iec = max_info.format(iec);
    assert_eq!(max_iec.suffix, "PiB");
    assert!(max_iec.value > 1024.0);

    let max_si = max_info.format(&SizeSystems::SI_SYSTEM);
    assert_eq!(max_si.suffix, "PB");
    assert!(max_si.value > 1000.0);
}

#[test]
fn test_file_size_info_format_with_custom_system_base_one() {
    // When base is 1.0: value >= base is always true for value >= 1.0.
    // Loop should terminate because index reaches MAX_PREFIX_INDEX (5).
    let prefixes = ["P0", "P1", "P2", "P3", "P4", "P5"];
    let sys = SizeSystem::new("base_one", 1.0, prefixes).unwrap();

    let info = FileSizeInfo::new(50);
    let formatted = info.format(&sys);
    assert_eq!(formatted.suffix, "P5");
    assert_eq!(formatted.value, 50.0);

    // If bytes is 0: 0.0 < 1.0, loop does not execute
    let info_zero = FileSizeInfo::new(0);
    let formatted_zero = info_zero.format(&sys);
    assert_eq!(formatted_zero.suffix, "P0");
    assert_eq!(formatted_zero.value, 0.0);
}

// =========================================================================
// FileSizeReport Unit, Corner & Edge Case Tests
// =========================================================================

#[test]
fn test_file_size_report_new_and_make_pair() {
    let info = FileSizeInfo::new(1000);
    let report = FileSizeReport::new(info, &SizeSystems::SI_SYSTEM, &SizeSystems::IEC);

    assert_eq!(report.info, info);
    assert_eq!(report.si_sys, &SizeSystems::SI_SYSTEM);
    assert_eq!(report.iec_sys, &SizeSystems::IEC);

    let pair = report.make_pair();
    assert_eq!(pair.si_size.value, 1.0);
    assert_eq!(pair.si_size.suffix, "KB");
    assert_eq!(pair.iec_size.value, 1000.0);
    assert_eq!(pair.iec_size.suffix, "B");
}

#[test]
fn test_file_size_report_display_structure() {
    let info = FileSizeInfo::new(1_048_576);
    let report = FileSizeReport::new(info, &SizeSystems::SI_SYSTEM, &SizeSystems::IEC);
    let output = report.to_string();

    assert!(output.starts_with("Bytes : 1048576\n"));
    assert!(output.contains("-----------------------------------------\n"));
    assert!(output.contains("SI                   IEC                 \n"));
}

#[test]
fn test_file_size_report_clone_and_equality() {
    let report1 = FileSizeReport::new(FileSizeInfo::new(123), &SizeSystems::SI_SYSTEM, &SizeSystems::IEC);
    let report2 = report1.clone();
    assert_eq!(report1, report2);
}
