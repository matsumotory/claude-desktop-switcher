//! `csw` の出力の受け手が先に閉じても、異常終了の表示を出さずに終わることを確かめる
//! (docs/SPECIFICATION.md §5.B)。
//!
//! Rust のプログラムは起動時に SIGPIPE を無視するので、閉じたパイプへ `println!` すると
//! 「failed printing to stdout: Broken pipe」で panic する。`csw profile list | head -1`
//! のような普通の使い方でこの表示が出ていた。

#![cfg(unix)]

use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};

#[test]
fn closed_stdout_reader_ends_csw_quietly_by_sigpipe() {
    // 本物の環境の一覧に触れないよう、HOME を空の一時フォルダに向ける。
    let home = std::env::temp_dir().join(format!("csw-broken-pipe-{}", std::process::id()));
    std::fs::create_dir_all(&home).expect("create a temporary HOME");

    // 読み手を先に閉じたパイプを stdout に渡す。最初の書き込みで EPIPE になる。
    let (reader, writer) = std::io::pipe().expect("create a pipe");
    drop(reader);

    let out = Command::new(env!("CARGO_BIN_EXE_csw"))
        .args(["profile", "list"])
        .env("HOME", &home)
        .stdout(Stdio::from(writer))
        .stderr(Stdio::piped())
        .output()
        .expect("run csw profile list");
    let _ = std::fs::remove_dir_all(&home);

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("panicked") && !stderr.contains("Broken pipe"),
        "csw must not report a panic when the reader is gone: {stderr}"
    );
    assert_eq!(
        out.status.signal(),
        Some(libc_sigpipe()),
        "csw must end by SIGPIPE like other Unix commands: {:?}",
        out.status
    );
}

/// SIGPIPE の番号。macOS と Linux のどちらでも 13。
fn libc_sigpipe() -> i32 {
    13
}
