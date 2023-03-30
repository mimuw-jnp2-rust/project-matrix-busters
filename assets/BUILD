load("@rules_rust//rust:defs.bzl", "rust_binary")
load("@bazel_skylib//rules:run_binary.bzl", "run_binary")

package(default_visibility = ["//visibility:public"])

rust_binary(
    name = "dft",
    srcs = ["dft.rs"],
    data = ["dft_source.json"],
    deps = ["@crate_index//:serde_json", "@crate_index//:serde"],
)

filegroup(
    name = "dft_source",
    srcs = ["dft_source.json"],
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

filegroup(
    name = "icon",
    srcs = ["icon.png"],
)
