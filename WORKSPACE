load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(
    name = "rules_rust",
    sha256 = "25209daff2ba21e818801c7b2dab0274c43808982d6aea9f796d899db6319146",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.21.1/rules_rust-v0.21.1.tar.gz"],
)


load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()

rust_register_toolchains(versions = ["1.69.0"], edition="2021")

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
            version = "0.21.0",
        ),
        "eframe": crate.spec(
            version = "0.21.3",
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
        "time": crate.spec(
            version = "0.3",
        ),
        "serde": crate.spec(
            version = "1.0",
            features = ["derive"],
        ),
        "serde_json": crate.spec(
            version = "1.0",
        ),
        "egui-toast": crate.spec(
            version = "0.6.0",
        ),
        "log": crate.spec(
            version = "0.4.17",
        ),
    },
    annotations = {
        "khronos_api": [
            crate.annotation(
                patches = [
                    # https://github.com/brendanzab/gl-rs/pull/536
                    "@//patches:khronos_api.patch",
                ],
                # The patch file was generated using git-diff and khronos_api
                # is a submodule, so we need to use -p2 to apply the patch.
                patch_args = ["-p2"],
            ),
        ],
    },
    render_config = render_config(
        default_package_name = ""
    ),
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()

