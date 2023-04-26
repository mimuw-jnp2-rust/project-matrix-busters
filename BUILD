load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@crate_index//:defs.bzl", "all_crate_deps")

package(default_visibility = ["//visibility:public"])

all_rs_sources = glob(["src/*.rs"])

fft_assets = ["//fourier:dft_result"]
common_assets = ["//assets:icon"]

common_rustc_flags = ["-O"]

common_deps = all_crate_deps(
    normal = True,
) + [
    "//locale"
]

fft_deps = [
    "//fourier:fourier_display",
]
clock_deps = [
    "//fractal_clock",
]

rust_binary(
    name = "jp2gmd",
    srcs = all_rs_sources,
    deps = common_deps,
    data = common_assets,
    rustc_flags = common_rustc_flags,
)

rust_binary(
    name = "jp2gmd_fft",
    srcs = all_rs_sources,
    deps = common_deps + fft_deps,
    data = fft_assets + common_assets,
    crate_features = ["fft"],
    rustc_flags = common_rustc_flags,
)

rust_binary(
    name = "jp2gmd_clock",
    srcs = all_rs_sources,
    deps = common_deps + clock_deps,
    data = common_assets,
    crate_features = ["clock"],
    rustc_flags = common_rustc_flags,
)
