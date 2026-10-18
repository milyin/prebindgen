use example_flat::*;

#[test]
fn entire_captured_surface_is_in_the_flat_subset() {
    let source = prebindgen::Source::new(PREBINDGEN_OUT_DIR);
    prebindgen_flat::Flat::<prebindgen_flat::InputPlan<()>, prebindgen_flat::OutputPlan<()>>::parse(source.items_all()).unwrap();
}

#[test]
fn reduced_caption_retains_its_text() {
    let caption = caption_new(u64::MAX, "héllo\0world", true);
    assert_eq!(caption.text, "héllo\0world");
    assert_eq!(caption.id, u64::MAX);
    assert!(caption.emphatic);
}
