extern crate cargo_readme;

use std::{path, fs};
use std::io::{Write};

fn main() {

    // Generate README.md with cargo_readme.

    let mut f = fs::File::open("src/lib.rs").unwrap();
    let content = cargo_readme::generate_readme(
        &path::PathBuf::from("./"),
        &mut f,
        None,
        false,
        false,
        false).unwrap();

    let mut f = fs::File::create("README.md").unwrap();
    f.write_all(content.as_bytes()).unwrap();
}