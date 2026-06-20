# Build and run the C micro-benchmark against the generated `perftest` C ABI.
#
# This compiles `c/perftest.c`, links it against the Rust cdylib built by cargo
# (`libperftest_c`), and runs it. The cargo build also regenerates the committed
# `include/perftest.h` from perftest-flat's `#[prebindgen]` surface.
#
# Usage:
#   cmake -S . -B build
#   cmake --build build                 # builds the Rust cdylib + the C program
#   cmake --build build --target run    # runs the benchmark
#
# Requires a Rust toolchain and a C compiler.

cmake_minimum_required(VERSION 3.16)
project(perftest_c LANGUAGES C)

set(CMAKE_C_STANDARD 11)

find_program(CARGO cargo REQUIRED)

# Workspace root is two levels up (examples/perftest-c/ -> repo root).
get_filename_component(_workspace "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)
set(_reldir "${_workspace}/target/release")
set(_dylib "${_reldir}/${CMAKE_SHARED_LIBRARY_PREFIX}perftest_c${CMAKE_SHARED_LIBRARY_SUFFIX}")

# Build the Rust cdylib (and regenerate include/perftest.h) via cargo.
add_custom_command(
  OUTPUT "${_dylib}"
  COMMAND ${CARGO} build --release -p perftest-c --manifest-path "${_workspace}/Cargo.toml"
  COMMENT "cargo build --release -p perftest-c (cdylib + regenerated header)"
  VERBATIM
)
add_custom_target(cargo_cdylib DEPENDS "${_dylib}")

add_executable(perftest_bench c/perftest.c)
add_dependencies(perftest_bench cargo_cdylib)
target_include_directories(perftest_bench PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/include")
target_link_libraries(perftest_bench PRIVATE "${_dylib}")
# Find the cdylib at runtime without installing it.
set_target_properties(perftest_bench PROPERTIES BUILD_RPATH "${_reldir}")

add_custom_target(run COMMAND perftest_bench DEPENDS perftest_bench)
