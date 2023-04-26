load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")
load("@bazel_skylib//rules:run_binary.bzl", "run_binary")

package(default_visibility = ["//visibility:public"])

### DFT generation

rust_binary(
    name = "dft",
    srcs = ["tools/dft.rs"],
    data = ["dft_source.json"],
    deps = [
        "@crate_index//:serde_json",
        "@crate_index//:serde",
        "@crate_index//:log"
    ],
)

run_binary(
    name = "dft_result",
    env = {
        "SOURCE_PATH": "$(location :dft_source.json)",
        "RESULT_PATH": "$(location :dft_result.json)",
        "EXPECTED_POINTS": "1000",
    },
    tool = ":dft",
    srcs = ["dft_source.json"],
    outs = ["dft_result.json"],
)

### DFT display

rust_library(
    name = "fourier_display",
    srcs = ["src/fourier.rs"],
    deps = [
        "@crate_index//:eframe",
        "@crate_index//:egui",
        "@crate_index//:itertools",
        "@crate_index//:serde",
        "@crate_index//:serde_json",
        "@crate_index//:anyhow",
    ],
)
