use perftest_flat::*;

#[test]
fn entire_captured_surface_is_in_the_flat_subset() {
    let source = prebindgen::Source::new(PREBINDGEN_OUT_DIR);
    let _: prebindgen_flat::Flat = prebindgen_flat::Flat::parse(source.items_all()).unwrap();
}

#[test]
fn reduced_records_retain_supported_values() {
    let id = vec![0, 1, 127, 128, 255];
    let blob = blob_value_echo(blob_value_new(-42, id.clone()));
    assert_eq!(blob.id, id);
    assert_eq!(blob.stamp.secs, -42);
    let payload = payload_new(i64::MIN, i32::MAX, -0.0, true);
    let projected = payload_to_struct(&payload);
    assert_eq!(projected.id, i64::MIN);
    assert_eq!(projected.seq, i32::MAX);
    assert_eq!(projected.value.to_bits(), (-0.0f64).to_bits());
    assert!(projected.flag);
    let report = report_new(2, 3.5, "héllo\0world".into());
    assert_eq!(report_to_struct(&report).label, "héllo\0world");
    assert_eq!(report_into_struct(report).label, "héllo\0world");
}
