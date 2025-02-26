// Copyright (c) 2024-2025 Zededa, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::env;
use std::process::Command;

fn main() {
    // if .git doesnt exist then we are not in a git repo
    // it may happen in container builds. do not set GIT_VERSION
    if !std::path::Path::new(".git").exists() {
        return;
    }

    // Get exact tag if it exists
    let exact_tag = Command::new("git")
        .args(["describe", "--tags", "--exact-match"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

    let dirty_descr = Command::new("git")
        .args(["describe", "--tags", "--dirty"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let git_info = exact_tag.unwrap_or(dirty_descr);

    // Set an environment variable in the Rust build output
    println!("cargo:rustc-env=GIT_VERSION={}", git_info);
}
