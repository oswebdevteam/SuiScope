use std::fs;
use std::path::Path;

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
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
    // Tell cargo to rerun this script if the frontend build changes
    println!("cargo:rerun-if-changed=../../frontend/out");

    let source_dir = Path::new("../../frontend/out");
    let dest_dir = Path::new("public");

    if source_dir.exists() {
        if let Err(e) = copy_dir_all(source_dir, dest_dir) {
            println!("cargo:warning=Failed to copy frontend files: {}", e);
        }
    } else if !dest_dir.exists() {
        // Fallback for local dev if they haven't run npm run build yet,
        // so rust-embed doesn't crash the cargo build.
        fs::create_dir_all(dest_dir).unwrap();
        fs::write(
            dest_dir.join("index.html"),
            "<html><body>Please run npm run build in the frontend directory first.</body></html>",
        )
        .unwrap();
    }
}
