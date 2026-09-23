//! Emit the build facts `--version` has to report, per `sw-checklist`.

use std::process::Command;

/// Run a command and take its first line, or "unknown".
fn first_line(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out = std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo");
    let commit = first_line("git", &["rev-parse", "--short", "HEAD"]);
    let host = first_line("hostname", &[]);
    let time = first_line("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"]);
    let version = env!("CARGO_PKG_VERSION");
    let text = format!(
        "/// Version, with the build facts sw-checklist requires.
pub const LONG_VERSION: &str = \"{version}
Copyright (c) 2026 Michael A Wright
License: MIT
Repository: https://github.com/software-wrighter-lab/sw-atlas
Build Host: {host}
Build Commit: {commit}
Build Time: {time}\";
"
    );
    std::fs::write(std::path::Path::new(&out).join("build_facts.rs"), text)
        .expect("build facts are writable");
}
