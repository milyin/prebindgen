//! Benchmarks for building the flat model from a captured item stream: type
//! normalization, classification of every item, and resolution of every type
//! reference against the declarations.
//!
//! The fixture is a synthetic `#[prebindgen]` surface shaped like a real FFI
//! crate (see `examples/perftest-flat`): `#[repr(C)]` structs, C-like enums,
//! opaque handle aliases, and the functions operating on them.

use prebindgen::{utils::write_to_jsonl_file, Record, RecordKind, Source, SourceLocation};
use prebindgen_flat::Flat;

fn main() {
    divan::main();
}

/// Number of FFI "modules" in the fixture; each contributes one struct, one
/// enum, one handle alias and several functions.
const SIZES: &[usize] = &[10, 100];

fn record(kind: RecordKind, name: String, content: String, line: usize) -> Record {
    let location = SourceLocation {
        file: "src/lib.rs".to_string(),
        line,
        column: 1,
        crate_name: None,
    };
    Record::new(kind, name, content, location, None)
}

fn make_records(n: usize) -> Vec<Record> {
    let mut records = Vec::with_capacity(n * 7);
    for i in 0..n {
        let line = i * 100;
        records.push(record(
            RecordKind::Struct,
            format!("Payload{i}"),
            format!(
                "#[repr(C)] #[derive(Clone, Debug, Default, PartialEq)] pub struct Payload{i} \
                 {{ pub id : i64, pub seq : i32, pub value : f64, pub flag : bool, \
                 pub label : Option < Box < String > >, }}"
            ),
            line,
        ));
        records.push(record(
            RecordKind::Enum,
            format!("Priority{i}"),
            format!(
                "#[repr(i32)] #[derive(Clone, Copy, Debug, PartialEq, Eq)] \
                 pub enum Priority{i} {{ Low = 0, Normal = 1, High = 2, }}"
            ),
            line + 10,
        ));
        records.push(record(
            RecordKind::TypeAlias,
            format!("Storage{i}"),
            format!("pub type Storage{i} = handles :: Storage{i};"),
            line + 20,
        ));
        records.push(record(
            RecordKind::Function,
            format!("storage{i}_new"),
            format!("pub fn storage{i}_new() -> Storage{i} {{}}"),
            line + 30,
        ));
        records.push(record(
            RecordKind::Function,
            format!("storage{i}_put"),
            format!(
                "pub fn storage{i}_put(storage : & mut Storage{i}, payload : Payload{i}) \
                 -> Result < (), String > {{}}"
            ),
            line + 40,
        ));
        records.push(record(
            RecordKind::Function,
            format!("storage{i}_get_all"),
            format!("pub fn storage{i}_get_all(storage : & Storage{i}) -> Vec < Payload{i} > {{}}"),
            line + 50,
        ));
        records.push(record(
            RecordKind::Function,
            format!("payload{i}_priority"),
            format!("pub fn payload{i}_priority(p : & Payload{i}) -> Priority{i} {{}}"),
            line + 60,
        ));
    }
    records
}

fn make_source_dir(n: usize) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join("crate_name.txt"), "bench_ffi").unwrap();
    std::fs::write(dir.path().join("features.txt"), "default\n").unwrap();
    write_to_jsonl_file(dir.path().join("default_0.jsonl"), &make_records(n)).unwrap();
    dir
}

/// Build the flat model from an already-loaded item stream.
#[divan::bench(args = SIZES)]
fn flat_build(bencher: divan::Bencher, n: usize) {
    let dir = make_source_dir(n);
    let items: Vec<_> = Source::builder(dir.path())
        .enable_target_filtering(None::<String>)
        .build()
        .items_all()
        .collect();
    bencher
        .with_inputs(|| items.clone())
        .bench_local_values(|items| Flat::builder().items(items).build().unwrap());
}

/// The whole build-script read path: load the directory, filter, and build
/// the flat model.
#[divan::bench(args = SIZES)]
fn flat_from_source_dir(bencher: divan::Bencher, n: usize) {
    let dir = make_source_dir(n);
    bencher.bench_local(|| {
        let source = Source::builder(divan::black_box(dir.path()))
            .enable_target_filtering(None::<String>)
            .build();
        Flat::builder().items(source.items_all()).build().unwrap()
    });
}
