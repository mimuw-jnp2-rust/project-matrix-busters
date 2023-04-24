load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")

git_repository(
    name = "rules_rust",
    commit = "8cc2191d2beb4d209eadee4eebaff80002604293",
    remote = "https://github.com/ZPP-This-is-fine/rules_rust.git",
)


load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()

rust_register_toolchains(versions = ["1.68.1"], edition="2021")

load("@rules_rust//crate_universe:defs.bzl", "crate", "crates_repository", "render_config")

crates_repository(
    name = "crate_index",
    cargo_lockfile = "//:Cargo.lock",
    lockfile = "//:Cargo.Bazel.lock",
    packages = {
        "itertools": crate.spec(
            version = "0.10.5",
        ),
        "egui": crate.spec(
            version = "0.20.1",
        ),
        "eframe": crate.spec(
            version = "0.20.1",
        ),
        "num-rational": crate.spec(
            version = "0.4.1",
        ),
        "num-traits": crate.spec(
            version = "0.2.15",
        ),
        "anyhow": crate.spec(
            version = "1.0.66",
        ),
        "image": crate.spec(
            version = "0.24.2",
        ),
        "lazy_static": crate.spec(
            version = "1.4.0",
        ),
        "arboard": crate.spec(
            version = "3.2.0",
        ),
        "clap": crate.spec(
            version = "4.1.1",
            features = ["derive"],
        ),
        "chrono": crate.spec(
            version = "0.4",
        ),
        "serde": crate.spec(
            version = "1.0",
            features = ["derive"],
        ),
        "serde_json": crate.spec(
            version = "1.0",
        ),
        "egui-toast": crate.spec(
            version = "0.5.0",
        ),
        "log": crate.spec(
            version = "0.4.17",
        ),
    },
    render_config = render_config(
        default_package_name = ""
    ),
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()

