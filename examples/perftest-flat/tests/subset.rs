use perftest_flat::*;

#[test]
fn entire_captured_surface_is_in_the_flat_subset() {
    let source = prebindgen::Source::new(PREBINDGEN_OUT_DIR);
    prebindgen_flat::Flat::<()>::parse(source.items_all()).unwrap();
}

#[test]
fn reduced_records_retain_supported_values() {
    let id = vec![0, 1, 127, 128, 255];
    assert_eq!(blob_value_echo(blob_value_new(id.clone())).id, id);
    let report = report_new("héllo\0world".into());
    assert_eq!(report_to_struct(&report).label, "héllo\0world");
    assert_eq!(report_into_struct(report).label, "héllo\0world");
}
