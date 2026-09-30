//! `csw --version` がリリースの版を出すことを確かめる (docs/SPECIFICATION.md §5.B)。
//!
//! release-please が上げるのは `Cargo.toml` の `workspace.package.version` だけなので、
//! 各クレートがその版を引き継いでいないと、`--version` と SBOM の版がリリースとずれる。

use std::process::Command;

/// リリースの版の正は release-please が管理する `.release-please-manifest.json`。
fn release_version() -> String {
    let manifest = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../.release-please-manifest.json"
    ));
    let json: serde_json::Value = serde_json::from_str(manifest).expect("manifest is JSON");
    json["."]
        .as_str()
        .expect("manifest has the root package version")
        .to_string()
}

#[test]
fn version_flag_prints_release_version() {
    let out = Command::new(env!("CARGO_BIN_EXE_csw"))
        .arg("--version")
        .output()
        .expect("run csw --version");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).expect("utf-8 output");
    assert_eq!(stdout.trim(), format!("csw {}", release_version()));
}

#[test]
fn every_crate_inherits_workspace_version() {
    let manifests = [
        (
            "core",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../core/Cargo.toml")),
        ),
        (
            "cli",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")),
        ),
        (
            "desktop",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../desktop/Cargo.toml"
            )),
        ),
    ];
    for (name, toml) in manifests {
        assert!(
            toml.lines().any(|l| l.trim() == "version.workspace = true"),
            "crates/{name}/Cargo.toml must inherit the workspace version"
        );
    }
}
