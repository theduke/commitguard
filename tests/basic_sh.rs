extern crate commitguard;

use std::io::Write;
use std::fs::File;

use commitguard::{Check, CommitGuard};

#[test]
fn basic_sh() {
    let guard = CommitGuard::new()
        .check(
            Check::new("cargo")
                .args(vec![
                    "fmt".to_string(),
                    "--".into(),
                    "--write-mode=overwrite".into(),
                ])
                .check_for_command("cargo-fmt")
                .require_command()
                .warn()
                .name("format")
                .silent(false)
                .install_instructions("cargo install rustfmt_nightly"),
        )
        .check(
            Check::new("cargo")
                .name("clippy")
                .args(vec!["clippy".to_string()])
                .check_for_command("cargo-clippy")
                .abort_immediate()
                .install_instructions("cargo install clippy"),
        );

    let output = guard.build_sh();
    println!("{}", output);

    let mut f = File::create("script.sh").unwrap();
    f.write_all(output.as_bytes()).unwrap();
}
