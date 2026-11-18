use prebindgen::{utils::read_jsonl_file, Source};
use prebindgen_proc_macro::prebindgen;

mod api {
    use super::prebindgen;

    #[prebindgen("api")]
    pub fn capture_group_probe() -> u8 {
        8
    }
}

mod api_v2 {
    use super::prebindgen;

    #[prebindgen("api_v2")]
    pub fn capture_group_probe() -> u16 {
        16
    }
}

mod api_200 {
    use super::prebindgen;

    #[prebindgen("api_200")]
    pub fn capture_group_probe() -> u32 {
        32
    }
}

#[test]
fn real_macro_keeps_complete_group_names() {
    // OUT_DIR is baked in by Cargo at compile time; use the public macro
    // instead of looking at Cargo's build directory at runtime.
    let path = std::path::Path::new(prebindgen_proc_macro::prebindgen_out_dir!());
    println!("capture fixture directory: {}", path.display());
    let mut groups = Vec::new();
    for entry in path.read_dir().unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        // The compatibility API still reads a bare Record and ignores group
        // metadata. Inspect the wire separately to assert the real macro emits it.
        let bare = read_jsonl_file(&path).unwrap();
        let json = std::fs::read_to_string(&path).unwrap();
        for (record, line) in bare.iter().zip(json.lines()) {
            if record.name == "capture_group_probe" {
                let row: serde_json::Value = serde_json::from_str(line).unwrap();
                groups.push(
                    row["capture_group"]
                        .as_str()
                        .expect("macro group metadata")
                        .to_owned(),
                );
            }
        }
    }
    groups.sort();
    groups.dedup();
    assert_eq!(groups, ["api", "api_200", "api_v2"]);

    let source = Source::new(path);
    for (group, expected) in [("api", "u8"), ("api_v2", "u16"), ("api_200", "u32")] {
        let outputs: Vec<_> = source
            .items_in_groups(&[group])
            .filter_map(|(item, _)| match item {
                syn::Item::Fn(f) if f.sig.ident == "capture_group_probe" => {
                    let syn::ReturnType::Type(_, ty) = f.sig.output else {
                        panic!("expected a return type");
                    };
                    Some(quote::quote!(#ty).to_string())
                }
                _ => None,
            })
            .collect();
        assert_eq!(outputs, [expected], "capture group {group}");
    }
    assert_eq!(
        (
            api::capture_group_probe(),
            api_v2::capture_group_probe(),
            api_200::capture_group_probe()
        ),
        (8, 16, 32)
    );
}
