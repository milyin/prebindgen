use example_flat::*;

#[test]
fn entire_captured_surface_is_in_the_flat_subset() {
    let source = prebindgen::Source::new(PREBINDGEN_OUT_DIR);
    prebindgen_flat::Flat::<()>::parse(source.items_all()).unwrap();
}

#[test]
fn reduced_caption_retains_its_text() {
    assert_eq!(caption_new("héllo\0world").text, "héllo\0world");
}
