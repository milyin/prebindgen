# prebindgen-project-root

A tiny utility crate that, when built inside a Cargo workspace, determines the workspace root using the `project-root` crate

- build.rs sets `PROJECT_ROOT` using `cargo:rustc-env`.
- `get_project_root()` returns that path at runtime.

If this crate is compiled from within `CARGO_HOME` (i.e., used as a dependency outside your workspace), the build will panic to avoid misconfiguration.
