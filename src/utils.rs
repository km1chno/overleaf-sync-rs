use std::path::Path;

pub fn path_to_str(path: &Path) -> &str {
    path.to_str().unwrap_or("INVALID PATH")
}
