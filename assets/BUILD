load("@rules_rust//rust:defs.bzl", "rust_binary")

package(default_visibility = ["//visibility:public"])

rust_binary(
    name = "dft",
    srcs = ["dft.rs"],
    data = ["dft_source.json"],
    deps = ["@crate_index//:serde_json", "@crate_index//:serde"],
)

genrule(
    name = "dft_result",
    srcs = [":dft"],
    outs = ["dft_result.json"],
    cmd = "echo $(location :dft)/dft_result.json > $@",
)

filegroup(
    name = "icon",
    srcs = ["icon.png"],
)
