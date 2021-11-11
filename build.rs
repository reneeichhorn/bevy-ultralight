use std::env;
use std::path::Path;
use std::{fs, io};

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn main() {
    let ultralight_dir = env::var("ULTRALIGHT").unwrap_or_else(|_| "C:/ultralight".to_string());
    println!(r"cargo:rustc-link-search={}/lib", ultralight_dir);

    let out_dir = env::var("OUT_DIR").unwrap();
    copy_dir_all(format!("{}/bin", ultralight_dir), out_dir).unwrap();
}
