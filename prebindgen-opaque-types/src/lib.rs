//! Build-time generator of **opaque `#[repr(C, align(_))]` counterpart structs**
//! for prebindgen's inline-by-value FFI (`lang::Cbindgen::value_opaque`).
//!
//! For each requested Rust type it emits an opaque struct of byte-identical size
//! and alignment, e.g.
//!
//! ```ignore
//! #[repr(C, align(8))]
//! pub struct z_zbytes_t { _0: [u8; 32] }
//! ```
//!
//! The size/alignment are obtained by **symbol extraction**: a tiny probe crate
//! that depends on the source crate is compiled *for the build's `$TARGET`* (so
//! the layout is correct under cross-compilation, where the host layout would be
//! wrong), exporting `#[no_mangle] static` `usize` constants whose values are then
//! read back from the compiled artifact with the `object` crate — no execution of
//! target code. The `lang::Cbindgen` `value_opaque` converters additionally emit
//! `const _` size/align equality asserts, so a wrong probe fails the *consumer's*
//! build (fail-closed) rather than corrupting memory.
//!
//! Usage from a consumer `build.rs`:
//! ```ignore
//! let cfg = Config { /* source crate, features, types, build_dir, … */ };
//! let generated = prebindgen_opaque_types::generate(&cfg)?;
//! // include! `generated` into the crate AND feed it to cbindgen.
//! ```

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

/// One type to probe and the opaque struct identifier to emit for it.
#[derive(Clone, Debug)]
pub struct OpaqueType {
    /// Rust path as visible from the probe crate, e.g. `"zenoh_flat::ZBytes"`.
    pub rust_path: String,
    /// Emitted opaque struct identifier, e.g. `"z_zbytes_t"`.
    pub opaque_name: String,
}

impl OpaqueType {
    pub fn new(rust_path: impl Into<String>, opaque_name: impl Into<String>) -> Self {
        Self {
            rust_path: rust_path.into(),
            opaque_name: opaque_name.into(),
        }
    }
}

/// Builder for an opaque-types generation run.
///
/// The source crate is identified solely by its **manifest directory** — pass
/// prebindgen's `MANIFEST_DIR` constant (which the source crate exports via
/// `prebindgen_proc_macro::manifest_dir!()`), so it works wherever the marked
/// crate actually lives — a path dependency anywhere, a git dependency, or
/// crates.io — without the consumer assuming any layout. The package name is read
/// from that directory's `Cargo.toml`.
///
/// Sensible defaults for the `build.rs` use case: `build_dir` =
/// `$OUT_DIR/opaque_probe`, `cargo_lock` = the destination workspace's `Cargo.lock`
/// (via [`prebindgen-project-root`]; the consumer must run
/// `cargo prebindgen-project-root install`), default features on. Typical use:
///
/// ```ignore
/// let opaque = prebindgen_opaque_types::OpaqueTypes::new(zenoh_flat::MANIFEST_DIR)
///     .features(zenoh_flat::FEATURES)            // prebindgen's feature string
///     .add(syn::parse_quote!(zenoh_flat::ZZBytes), syn::parse_quote!(z_zbytes_t))
///     .generate()?;
/// ```
#[derive(Clone, Debug)]
pub struct OpaqueTypes {
    source_manifest_dir: PathBuf,
    features: Vec<String>,
    no_default_features: bool,
    types: Vec<OpaqueType>,
    cargo_lock: PathBuf,
    build_dir: Option<PathBuf>,
}

impl OpaqueTypes {
    /// Start a run probing types from the source crate located at
    /// `source_manifest_dir` — pass the source crate's `MANIFEST_DIR` constant.
    /// The package name is read from `<source_manifest_dir>/Cargo.toml`.
    pub fn new(source_manifest_dir: impl Into<PathBuf>) -> Self {
        let build_dir =
            std::env::var_os("OUT_DIR").map(|o| PathBuf::from(o).join("opaque_probe"));
        Self {
            source_manifest_dir: source_manifest_dir.into(),
            features: Vec::new(),
            no_default_features: false,
            types: Vec::new(),
            cargo_lock: default_cargo_lock(),
            build_dir,
        }
    }

    /// Mirror the source crate's enabled features from prebindgen's `FEATURES`
    /// string — whitespace-separated `"crate/feature"` items; pass the crate's
    /// `FEATURES` const directly. Because that is the *complete* resolved set, this
    /// also switches the probe to `--no-default-features` for an exact mirror.
    pub fn features(mut self, prebindgen_features: &str) -> Self {
        self.features = prebindgen_features
            .split_whitespace()
            .filter_map(|qf| qf.rsplit_once('/').map(|(_, f)| f.to_string()))
            .collect();
        self.no_default_features = true;
        self
    }

    /// Override whether the source crate's default features are enabled. Defaults
    /// to enabled, unless [`Self::features`] already set the complete resolved set.
    pub fn default_features(mut self, enabled: bool) -> Self {
        self.no_default_features = !enabled;
        self
    }

    /// Add a type to probe: its Rust path as seen from the probe crate (e.g.
    /// `syn::parse_quote!(zenoh_flat::ZZBytes)`) and the opaque counterpart
    /// identifier to emit (e.g. `syn::parse_quote!(z_zbytes_t)`). Takes `syn::Type`
    /// (Rust code) like [`crate`]'s sibling builders (`Cbindgen::value_opaque`).
    pub fn add(mut self, rust_ty: syn::Type, opaque_ty: syn::Type) -> Self {
        use quote::ToTokens;
        self.types.push(OpaqueType::new(
            rust_ty.to_token_stream().to_string(),
            opaque_ty.to_token_stream().to_string(),
        ));
        self
    }

    /// Override the `Cargo.lock` copied into the probe crate (default: the
    /// destination workspace's lock, located via [`prebindgen-project-root`]).
    pub fn cargo_lock(mut self, path: impl Into<PathBuf>) -> Self {
        self.cargo_lock = path.into();
        self
    }

    /// Override the probe build directory (default: `$OUT_DIR/opaque_probe`).
    pub fn build_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.build_dir = Some(path.into());
        self
    }

    /// Probe each type's `$TARGET` size/alignment and return the generated
    /// opaque-struct Rust source (also written to `<build_dir>/opaque_types.rs`).
    pub fn generate(&self) -> Result<String> {
        let build_dir = self
            .build_dir
            .clone()
            .ok_or_else(|| anyhow!("build_dir not set and OUT_DIR is unavailable"))?;
        let target = std::env::var("TARGET").unwrap_or_default();
        write_probe_crate(self, &build_dir)?;
        let rlib = build_probe(self, &build_dir, &target)?;
        let data =
            std::fs::read(&rlib).with_context(|| format!("reading {}", rlib.display()))?;

        let mut out = String::from(
            "// @generated by prebindgen-opaque-types — do not edit.\n\
             // Opaque #[repr(C, align)] counterparts for value_opaque inline-by-value FFI.\n\n",
        );
        for t in &self.types {
            let size = read_symbol_usize(&data, &sym_name("SIZE", &t.opaque_name))
                .with_context(|| format!("probing size of `{}`", t.rust_path))?;
            let align = read_symbol_usize(&data, &sym_name("ALIGN", &t.opaque_name))
                .with_context(|| format!("probing align of `{}`", t.rust_path))?;
            out.push_str(&render_opaque(&t.opaque_name, size, align));
        }
        let out_path = build_dir.join("opaque_types.rs");
        std::fs::write(&out_path, &out)
            .with_context(|| format!("writing {}", out_path.display()))?;
        Ok(out)
    }
}

const PROBE_CRATE: &str = "prebindgen_opaque_probe";

/// Symbol name for a probed quantity (`"SIZE"` / `"ALIGN"`) of an opaque type.
fn sym_name(kind: &str, opaque_name: &str) -> String {
    format!("PREBINDGEN_{kind}_{opaque_name}")
}

/// Render the probe crate's `lib.rs`: one `#[no_mangle] static usize` per quantity.
pub fn render_probe_lib(types: &[OpaqueType]) -> String {
    let mut s = String::from(
        "// @generated probe crate for prebindgen-opaque-types — do not edit.\n\
         #![allow(non_upper_case_globals, dead_code)]\n",
    );
    for t in types {
        let size_sym = sym_name("SIZE", &t.opaque_name);
        let align_sym = sym_name("ALIGN", &t.opaque_name);
        let path = &t.rust_path;
        s.push_str(&format!(
            "#[no_mangle]\n#[used]\npub static {size_sym}: usize = ::core::mem::size_of::<{path}>();\n\
             #[no_mangle]\n#[used]\npub static {align_sym}: usize = ::core::mem::align_of::<{path}>();\n",
        ));
    }
    s
}

/// Render one opaque counterpart struct.
///
/// Only the `#[repr(C, align)]` struct is emitted. The `prebindgen::Transmute`
/// impl (the unsafe rust<->opaque glue) is generated by `lang::Cbindgen` for the
/// `value_opaque` pair; the consumer supplies only the `prebindgen::Gravestone`
/// *logic* (orphan-rule legal because the opaque type is local to the consumer):
///
/// ```ignore
/// impl ::prebindgen::Gravestone for z_zbytes_t {
///     fn rust_gravestone() -> zenoh_flat::ZZBytes { zenoh_flat::ZZBytes::default() }
///     fn rust_is_gravestone(r: &zenoh_flat::ZZBytes) -> bool { r.is_empty() }
/// }
/// ```
pub fn render_opaque(opaque_name: &str, size: usize, align: usize) -> String {
    // The byte field is `pub` so cbindgen emits a *complete* C type
    // (`{ uint8_t _0[N]; }`) — required to pass the opaque by value. (It carries
    // no real fields to access; it is only ever transmuted to/from the Rust type.)
    format!(
        "#[repr(C, align({align}))]\n#[allow(non_camel_case_types)]\n\
         pub struct {opaque_name} {{\n    pub _0: [u8; {size}],\n}}\n\n"
    )
}

/// The **destination project's** `Cargo.lock`, so the probe resolves dependencies
/// identically to the cdylib build.
///
/// The workspace root comes from [`prebindgen_project_root::get_project_root`] —
/// correct even for a consumer installed from crates.io, because
/// `prebindgen-project-root` is patched into the *destination* workspace, so every
/// copy in the graph (including this library's dependency on it) resolves to that
/// member copy and reports the destination root. The consumer must have run
/// `cargo prebindgen-project-root install` (otherwise `get_project_root` panics
/// with guidance — there is intentionally no silent fallback).
fn default_cargo_lock() -> PathBuf {
    prebindgen_project_root::get_project_root().join("Cargo.lock")
}

/// Read the `[package].name` of the crate whose manifest dir is `manifest_dir`.
/// It is the `[dependencies]` key the probe must use for a path dependency.
fn read_package_name(manifest_dir: &Path) -> Result<String> {
    let manifest_path = manifest_dir.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("reading {}", manifest_path.display()))?;
    let table: toml::Table = text
        .parse()
        .with_context(|| format!("parsing {}", manifest_path.display()))?;
    table
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(str::to_string)
        .ok_or_else(|| anyhow!("no `[package].name` (string) in {}", manifest_path.display()))
}

/// Write the probe crate (`Cargo.toml` + `src/lib.rs`, and `Cargo.lock` if given).
fn write_probe_crate(b: &OpaqueTypes, build_dir: &Path) -> Result<()> {
    let src = build_dir.join("src");
    std::fs::create_dir_all(&src)
        .with_context(|| format!("creating probe src dir {}", src.display()))?;
    let package = read_package_name(&b.source_manifest_dir)?;

    let features_toml = if b.features.is_empty() {
        String::new()
    } else {
        let list = b
            .features
            .iter()
            .map(|f| format!("\"{f}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!(", features = [{list}]")
    };
    let manifest = format!(
        "[package]\nname = \"{PROBE_CRATE}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\
         publish = false\n\n[lib]\ncrate-type = [\"lib\"]\n\n[dependencies]\n\
         {pkg} = {{ path = {path:?}, default-features = {dflt}{features} }}\n\n\
         [workspace]\n",
        pkg = package,
        path = b.source_manifest_dir,
        dflt = !b.no_default_features,
        features = features_toml,
    );
    std::fs::write(build_dir.join("Cargo.toml"), manifest)?;
    std::fs::write(src.join("lib.rs"), render_probe_lib(&b.types))?;
    if b.cargo_lock.exists() {
        let _ = std::fs::copy(&b.cargo_lock, build_dir.join("Cargo.lock"));
    }
    Ok(())
}

/// Build the probe crate for `$TARGET` and return the path to its rlib.
fn build_probe(_b: &OpaqueTypes, build_dir: &Path, target: &str) -> Result<PathBuf> {
    let mut cmd = std::process::Command::new(std::env::var("CARGO").unwrap_or("cargo".into()));
    cmd.current_dir(build_dir)
        .arg("build")
        .arg("--offline")
        .arg("--message-format=json-render-diagnostics")
        .arg("--manifest-path")
        .arg(build_dir.join("Cargo.toml"));
    if !target.is_empty() {
        cmd.arg("--target").arg(target);
    }
    // Isolate the probe's target dir from the consumer's (avoid lock contention).
    cmd.arg("--target-dir").arg(build_dir.join("target"));
    let out = cmd.output().context("spawning cargo for the probe crate")?;
    if !out.status.success() {
        bail!(
            "probe build failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    // Parse cargo JSON for the probe crate's rlib artifact.
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut rlib: Option<PathBuf> = None;
    for line in stdout.lines() {
        // Minimal JSON scan (avoid a serde dep): look for the probe's artifact line.
        if line.contains("\"compiler-artifact\"") && line.contains(PROBE_CRATE) {
            if let Some(p) = extract_first_rlib(line) {
                rlib = Some(PathBuf::from(p));
            }
        }
    }
    rlib.ok_or_else(|| anyhow!("probe rlib artifact not found in cargo output"))
}

/// Pull the first `.rlib` path out of a cargo `compiler-artifact` JSON line.
fn extract_first_rlib(line: &str) -> Option<String> {
    // `"filenames":["...rlib", ...]` — find the first quoted token ending in .rlib.
    // Cargo emits JSON, so on Windows the path's backslashes arrive escaped
    // (`C:\\…\\libprobe.rlib`); unescape the standard JSON sequences before use.
    let idx = line.find("\"filenames\"")?;
    let rest = &line[idx..];
    for tok in rest.split('"') {
        if tok.ends_with(".rlib") {
            return Some(json_unescape(tok));
        }
    }
    None
}

/// Minimal JSON string unescaping for the path tokens cargo emits (`\\`, `\"`,
/// `\/`). Sufficient for filesystem paths; not a general JSON unescaper.
fn json_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('/') => out.push('/'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Read the `usize` value of a `#[no_mangle] static` named `sym` from a compiled
/// rlib/object (an `ar` archive of object files). Tries the bare name and the
/// Mach-O `_`-prefixed variant.
pub fn read_symbol_usize(artifact: &[u8], sym: &str) -> Result<usize> {
    use object::{Object, ObjectSection, ObjectSymbol};

    let with_underscore = format!("_{sym}");
    let matches = |name: &str| name == sym || name == with_underscore;

    let read_from = |obj: &object::File| -> Option<usize> {
        let ptr_bytes = if obj.is_64() { 8 } else { 4 };
        let s = obj.symbols().find(|s| s.name().map(matches).unwrap_or(false))?;
        let sec = obj.section_by_index(s.section_index()?).ok()?;
        let data = sec.data().ok()?;
        let off = s.address().checked_sub(sec.address())? as usize;
        let bytes = data.get(off..off + ptr_bytes)?;
        let mut v = [0u8; 8];
        v[..ptr_bytes].copy_from_slice(bytes);
        Some(u64::from_le_bytes(v) as usize)
    };

    // rlib / .a is an ar archive of object members; plain .o parses directly.
    if let Ok(archive) = object::read::archive::ArchiveFile::parse(artifact) {
        for member in archive.members() {
            let member = member.map_err(|e| anyhow!("archive member: {e}"))?;
            let data = member
                .data(artifact)
                .map_err(|e| anyhow!("archive member data: {e}"))?;
            if let Ok(obj) = object::File::parse(data) {
                if let Some(v) = read_from(&obj) {
                    return Ok(v);
                }
            }
        }
    } else if let Ok(obj) = object::File::parse(artifact) {
        if let Some(v) = read_from(&obj) {
            return Ok(v);
        }
    }
    bail!("symbol `{sym}` not found in probe artifact")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_lib_emits_size_and_align_statics() {
        let types = vec![OpaqueType::new("zenoh_flat::ZBytes", "z_zbytes_t")];
        let s = render_probe_lib(&types);
        assert!(s.contains(
            "pub static PREBINDGEN_SIZE_z_zbytes_t: usize = ::core::mem::size_of::<zenoh_flat::ZBytes>();"
        ));
        assert!(s.contains(
            "pub static PREBINDGEN_ALIGN_z_zbytes_t: usize = ::core::mem::align_of::<zenoh_flat::ZBytes>();"
        ));
        assert!(s.contains("#[no_mangle]") && s.contains("#[used]"));
    }

    #[test]
    fn features_string_parsed_and_disables_defaults() {
        let b = OpaqueTypes::new("/tmp/src")
            .features("zenoh-flat/unstable zenoh-flat/shared-memory zenoh-flat/transport_tcp")
            .add(syn::parse_quote!(zenoh_flat::ZZBytes), syn::parse_quote!(z_zbytes_t));
        assert_eq!(
            b.features,
            vec!["unstable", "shared-memory", "transport_tcp"]
        );
        assert!(b.no_default_features, "complete FEATURES set ⇒ no_default_features");
        assert_eq!(b.types.len(), 1);
        assert_eq!(b.types[0].opaque_name, "z_zbytes_t");
    }

    #[test]
    fn opaque_struct_renders_repr_c_align() {
        let s = render_opaque("z_zbytes_t", 32, 8);
        assert!(s.contains("#[repr(C, align(8))]"));
        assert!(s.contains("pub struct z_zbytes_t"));
        assert!(s.contains("pub _0: [u8; 32]"));
    }

    #[test]
    fn rlib_artifact_path_parsed_from_cargo_json() {
        let line = r#"{"reason":"compiler-artifact","package_id":"prebindgen_opaque_probe 0.0.0","filenames":["/tmp/t/target/debug/deps/libprebindgen_opaque_probe-abc.rlib"],"executable":null}"#;
        assert_eq!(
            extract_first_rlib(line).as_deref(),
            Some("/tmp/t/target/debug/deps/libprebindgen_opaque_probe-abc.rlib")
        );
    }

    #[test]
    fn rlib_artifact_path_windows_backslashes_unescaped() {
        // Cargo JSON escapes Windows backslashes as `\\`.
        let line = r#"{"reason":"compiler-artifact","filenames":["C:\\proj\\target\\debug\\deps\\libprebindgen_opaque_probe-abc.rlib"]}"#;
        assert_eq!(
            extract_first_rlib(line).as_deref(),
            Some(r"C:\proj\target\debug\deps\libprebindgen_opaque_probe-abc.rlib")
        );
    }
}
