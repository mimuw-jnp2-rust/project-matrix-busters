load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

package(default_visibility = ["//visibility:public"])

rust_library(
    name = "locale",
    srcs = ["src/locale.rs"],
    deps = [
        "@crate_index//:lazy_static",
    ],
)

rust_test(
    name = "locale_test",
    crate = ":locale",
)
