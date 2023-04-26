load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@crate_index//:defs.bzl", "all_crate_deps")

package(default_visibility = ["//visibility:public"])

all_rs_sources = glob(["src/*.rs"])
fft_rs_sources = ["src/fourier.rs"]
clock_rs_sources = ["src/fractal_clock.rs"]
common_rs_sources = [s for s in all_rs_sources if s not in fft_rs_sources + clock_rs_sources]

fft_assets = ["//assets:dft_result"]
common_assets = ["//assets:icon"]

common_rustc_flags = ["-O"]

common_deps = all_crate_deps(
    normal = True,
)

rust_binary(
    name = "jp2gmd",
    srcs = common_rs_sources,
    deps = common_deps,
    data = common_assets,
    rustc_flags = common_rustc_flags,
)

rust_binary(
    name = "jp2gmd_fft",
    srcs = fft_rs_sources + common_rs_sources,
    deps = common_deps,
    data = fft_assets + common_assets,
    crate_features = ["fft"],
    rustc_flags = common_rustc_flags,
)

rust_binary(
    name = "jp2gmd_clock",
    srcs = clock_rs_sources + common_rs_sources,
    deps = common_deps,
    data = common_assets,
    crate_features = ["clock"],
    rustc_flags = common_rustc_flags,
)
