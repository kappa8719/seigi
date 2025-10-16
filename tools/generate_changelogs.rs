#!/usr/bin/env -S cargo -Zscript

use std::process::{Command, Stdio};

fn main() {
    let paths = [".", "demo", "seigi_focus", "seigi_form", "seigi_toast"];
    for path in paths.iter() {
        println!("Generate CHANGELOG.md for directory {path}");

        Command::new("git")
            .args(["cliff", "--bump", "-o"])
            .current_dir(path)
            .stdout(Stdio::piped())
            .status()
            .expect(format!("failed to generate changelog for directory {path}").as_str());
    }
}
