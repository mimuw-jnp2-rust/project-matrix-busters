load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test")

package(default_visibility = ["//visibility:public"])

all_rs_sources = glob(["src/*.rs"])
fft_rs_sources = ["src/furier.rs"]
clock_rs_sources = ["src/fractal_clock.rs"]
common_rs_sources = [s for s in all_rs_sources if s not in fft_rs_sources + clock_rs_sources]

fft_assets = ["assets/dft_andrzej.json"]
common_assets = ["assets/icon.png"]

common_rustc_flags = ["-O"]

common_deps = [
        "@crate_index//:itertools",
        "@crate_index//:egui",
        "@crate_index//:eframe",
        "@crate_index//:num-rational",
        "@crate_index//:num-traits",
        "@crate_index//:anyhow",
        "@crate_index//:image",
        "@crate_index//:lazy_static",
        "@crate_index//:arboard",
        "@crate_index//:clap",
        "@crate_index//:chrono",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:egui-toast",
]

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
