//! Benchmarks for the read side of the pipeline: loading the JSONL records
//! `#[prebindgen]` captured, parsing them back into `syn` items, and streaming
//! them through the cfg filter.
//!
//! The fixture is a synthetic prebindgen output directory shaped like the one
//! the proc-macro writes for a real FFI crate (see `examples/perftest-flat`):
//! structs, enums, handle aliases and functions, a share of them feature- or
//! target-gated.

use std::path::Path;

use prebindgen::{
    utils::{read_jsonl_file, write_to_jsonl_file},
    Record, RecordKind, Source, SourceLocation, TargetTriple,
};

fn main() {
    divan::main();
}

/// Number of FFI "modules" in the fixture; each contributes one struct, one
/// enum, one handle alias and several functions.
const SIZES: &[usize] = &[10, 100];

const TARGET: &str = "x86_64-unknown-linux-gnu";

fn location(line: usize) -> SourceLocation {
    SourceLocation {
        file: "src/lib.rs".to_string(),
        line,
        column: 1,
        crate_name: None,
    }
}

/// Records shaped like the proc-macro's output for `n` FFI modules.
fn make_records(n: usize) -> Vec<Record> {
    let mut records = Vec::with_capacity(n * 7);
    let mut line = 1;
    let mut push = |records: &mut Vec<Record>, kind, name: String, content: String, cfg| {
        records.push(Record::new(kind, name, content, location(line), cfg));
        line += 10;
    };
    for i in 0..n {
        let feature_cfg = (i % 4 == 1).then(|| "feature = \"extra\"".to_string());
        let missing_cfg = (i % 4 == 2).then(|| "feature = \"missing\"".to_string());
        let target_cfg = (i % 4 == 3).then(|| "target_os = \"linux\"".to_string());
        push(
            &mut records,
            RecordKind::Struct,
            format!("Payload{i}"),
            format!(
                "#[doc = \" Payload number {i}.\"] #[repr(C)] \
                 #[derive(Clone, Debug, Default, PartialEq)] pub struct Payload{i} \
                 {{ pub id : i64, pub seq : i32, pub value : f64, pub flag : bool, \
                 pub label : Option < Box < String > >, }}"
            ),
            feature_cfg.clone(),
        );
        push(
            &mut records,
            RecordKind::Enum,
            format!("Priority{i}"),
            format!(
                "#[repr(i32)] #[derive(Clone, Copy, Debug, PartialEq, Eq)] \
                 pub enum Priority{i} {{ Low = 0, Normal = 1, High = 2, }}"
            ),
            None,
        );
        push(
            &mut records,
            RecordKind::TypeAlias,
            format!("Storage{i}"),
            format!("pub type Storage{i} = handles :: Storage{i};"),
            target_cfg.clone(),
        );
        push(
            &mut records,
            RecordKind::Function,
            format!("storage{i}_new"),
            format!("#[doc = \" Create a storage.\"] pub fn storage{i}_new() -> Storage{i} {{}}"),
            target_cfg,
        );
        push(
            &mut records,
            RecordKind::Function,
            format!("storage{i}_put"),
            format!(
                "pub fn storage{i}_put(storage : & mut Storage{i}, payload : Payload{i}) \
                 -> Result < (), String > {{}}"
            ),
            missing_cfg,
        );
        push(
            &mut records,
            RecordKind::Function,
            format!("storage{i}_get"),
            format!(
                "pub fn storage{i}_get(storage : & Storage{i}, index : usize) \
                 -> Option < Payload{i} > {{}}"
            ),
            feature_cfg,
        );
        push(
            &mut records,
            RecordKind::Function,
            format!("payload{i}_priority"),
            format!("pub fn payload{i}_priority(p : & Payload{i}) -> Priority{i} {{}}"),
            None,
        );
    }
    records
}

/// Target filtering resolves the triple by spawning `rustc --print cfg` once
/// and caching the result process-wide; resolve it up front so the
/// measurement covers only the filtering itself.
fn filtered_source(dir: &Path) -> Source {
    TargetTriple::parse(TARGET).expect("rustc --print cfg");
    Source::builder(dir)
        .enable_target_filtering(Some(TARGET))
        .build()
}

/// A prebindgen output directory for `n` FFI modules, split over two groups
/// and several files the way concurrent proc-macro invocations write it.
fn make_source_dir(n: usize) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    write_source_dir(dir.path(), n);
    dir
}

fn write_source_dir(dir: &Path, n: usize) {
    std::fs::write(dir.join("crate_name.txt"), "bench_ffi").unwrap();
    std::fs::write(dir.join("features.txt"), "default\nextra\n").unwrap();
    let records = make_records(n);
    for (chunk_index, chunk) in records.chunks(16).enumerate() {
        let group = if chunk_index % 2 == 0 {
            "default"
        } else {
            "structs"
        };
        let file = dir.join(format!("{group}_{chunk_index}.jsonl"));
        write_to_jsonl_file(file, chunk).unwrap();
    }
}

#[divan::bench(args = SIZES)]
fn record_to_jsonl(bencher: divan::Bencher, n: usize) {
    let records = make_records(n);
    bencher.bench_local(|| {
        divan::black_box(&records)
            .iter()
            .map(|r| r.to_jsonl_string().unwrap().len())
            .sum::<usize>()
    });
}

#[divan::bench(args = SIZES)]
fn read_jsonl(bencher: divan::Bencher, n: usize) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("default_0.jsonl");
    write_to_jsonl_file(&file, &make_records(n)).unwrap();
    bencher.bench_local(|| read_jsonl_file(divan::black_box(&file)).unwrap());
}

/// Read every group and parse each record back into a `syn::Item`.
#[divan::bench(args = SIZES)]
fn source_load(bencher: divan::Bencher, n: usize) {
    let dir = make_source_dir(n);
    bencher.bench_local(|| {
        Source::builder(divan::black_box(dir.path()))
            .enable_target_filtering(None::<String>)
            .build()
    });
}

/// Stream every item through the feature + target cfg filter.
#[divan::bench(args = SIZES)]
fn source_items_all_filtered(bencher: divan::Bencher, n: usize) {
    let dir = make_source_dir(n);
    let source = filtered_source(dir.path());
    bencher.bench_local(|| divan::black_box(&source).items_all().count());
}

/// Stream every item without any filtering (pass-through path).
#[divan::bench(args = SIZES)]
fn source_items_all_unfiltered(bencher: divan::Bencher, n: usize) {
    let dir = make_source_dir(n);
    let source = Source::builder(dir.path())
        .enable_feature_filtering(None::<String>)
        .enable_target_filtering(None::<String>)
        .build();
    bencher.bench_local(|| divan::black_box(&source).items_all().count());
}

#[divan::bench(args = SIZES)]
fn source_items_in_groups(bencher: divan::Bencher, n: usize) {
    let dir = make_source_dir(n);
    let source = filtered_source(dir.path());
    bencher.bench_local(|| {
        divan::black_box(&source)
            .items_in_groups(&["structs"])
            .count()
    });
}
