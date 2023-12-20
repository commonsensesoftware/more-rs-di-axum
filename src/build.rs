use std::env::var;
use std::error::Error;
use std::fs::copy;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let cwd = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap());
    let root = cwd.parent().unwrap();
    let readme = root.join("README.md");

    if readme.exists() {
        copy(root.join("README.md"), cwd.join("README.md"))?;
    }

    Ok(())
}
