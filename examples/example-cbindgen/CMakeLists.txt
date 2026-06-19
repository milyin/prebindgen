# Build `example-cbindgen` for BOTH supported target architectures via cargo.
#
# This demonstrates that prebindgen's `#[prebindgen]` `cfg` handling generates
# *different* code per target: each `cargo build --target <triple>` runs the
# build script, which regenerates the per-target Rust FFI layer and C header under
#   generated/example_flat_<arch>.rs
#   include/example_flat_<arch>.h
# Compare the two `include/example_flat_*.h` (or run the `show_generated_diff`
# target) to see `foo_t`'s fields and `inside_foo_t`'s discriminants change with
# the target architecture.
#
# Requirements: a Rust toolchain with both targets installed, e.g.
#   rustup target add x86_64-apple-darwin aarch64-apple-darwin            # macOS
#   rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu  # Linux
#
# Usage:
#   cmake -S . -B build
#   cmake --build build                                  # builds both targets
#   cmake --build build --target show_generated_diff     # show the per-target diff
#
# Options:
#   -DEXAMPLE_TARGETS="triple-a;triple-b"   override the target triples
#   -DEXAMPLE_FEATURES=unstable             forward cargo features

cmake_minimum_required(VERSION 3.16)
project(example_cbindgen_multitarget LANGUAGES NONE)

find_program(CARGO cargo REQUIRED)

# The two "supported targets" of the cfg demonstration (x86_64 + aarch64).
if(APPLE)
  set(_default_targets "x86_64-apple-darwin;aarch64-apple-darwin")
else()
  set(_default_targets "x86_64-unknown-linux-gnu;aarch64-unknown-linux-gnu")
endif()
set(EXAMPLE_TARGETS "${_default_targets}" CACHE STRING "Rust target triples to build")

# Optional cargo features, e.g. -DEXAMPLE_FEATURES=unstable.
set(EXAMPLE_FEATURES "" CACHE STRING "comma-separated cargo features to enable")
set(_feature_args "")
if(EXAMPLE_FEATURES)
  set(_feature_args --features ${EXAMPLE_FEATURES})
endif()

# Workspace manifest (repo root is two levels up from examples/example-cbindgen/).
get_filename_component(_workspace "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)
set(_manifest "${_workspace}/Cargo.toml")

# One cargo build per target architecture. cargo is incremental, so re-running is
# cheap; each run (re)generates that target's committed FFI file + header.
add_custom_target(build_all_targets ALL)
foreach(_triple IN LISTS EXAMPLE_TARGETS)
  add_custom_command(
    TARGET build_all_targets POST_BUILD
    COMMAND ${CARGO} build --manifest-path "${_manifest}"
            -p example-cbindgen --target ${_triple} ${_feature_args}
    COMMENT "cargo build example-cbindgen for ${_triple} (generates per-target FFI + header)"
    VERBATIM
  )
endforeach()

# Show that the two architectures produced different generated headers (the point
# of the demonstration). `diff` exits non-zero when files differ, so swallow it.
add_custom_target(show_generated_diff
  COMMAND ${CMAKE_COMMAND} -E echo "Per-target generated headers differ by target_arch:"
  COMMAND sh -c "diff '${CMAKE_CURRENT_SOURCE_DIR}/include/example_flat_x86_64.h' '${CMAKE_CURRENT_SOURCE_DIR}/include/example_flat_aarch64.h' || true"
  DEPENDS build_all_targets
  VERBATIM
)
