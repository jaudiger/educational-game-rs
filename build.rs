use std::env;
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

fn main() {
    let content_directory = Path::new("assets/content/themes");
    println!("cargo:rerun-if-changed={}", content_directory.display());

    let mut filenames = fs::read_dir(content_directory)
        .expect("cannot read theme content directory")
        .map(|entry| entry.expect("cannot read theme content entry").path())
        .filter(|path| path.extension() == Some(OsStr::new("json")))
        .map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .expect("theme filename must be valid UTF-8")
                .to_owned()
        })
        .collect::<Vec<_>>();
    filenames.sort();

    let mut source = String::from("pub const THEME_DOCUMENTS: &[(&str, &str)] = &[\n");
    for filename in filenames {
        println!("cargo:rerun-if-changed=assets/content/themes/{filename}");
        let _ = writeln!(
            source,
            "    ({filename:?}, include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/content/themes/\", {filename:?}))),"
        );
    }
    source.push_str("];\n");

    let output_directory = env::var_os("OUT_DIR").expect("OUT_DIR is not set");
    fs::write(
        Path::new(&output_directory).join("embedded_content.rs"),
        source,
    )
    .expect("cannot write embedded content manifest");
}
