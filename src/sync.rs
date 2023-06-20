use anyhow::{Context, Result};
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

pub fn find_in_ancestors(start_dir: &Path, filename: &str) -> Option<PathBuf> {
    for dir in start_dir.ancestors() {
        let file = dir.join(filename);
        if file.exists() {
            return Some(file);
        }
    }
    None
}

pub fn find_corresponding_path(start_dir: &Path, filename: &str) -> Result<PathBuf> {
    let ancestor_path = find_in_ancestors(start_dir, filename)
        .with_context(|| format!("correspondence path file: {filename} not found in ancestors"))?;
    let corr_ancestor_path = read_to_string(&ancestor_path)?.trim().to_string();
    let suffix = start_dir
        .strip_prefix(
            ancestor_path
                .parent()
                .expect("correspondence path has parent"),
        )
        .expect("correspondence path is in ancestory");
    Ok(Path::new(&corr_ancestor_path).join(suffix))
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;

    use super::*;

    #[test]
    fn test_find_in_ancestors() {
        let path = find_in_ancestors(&current_dir().unwrap(), "Cargo.toml");
        assert!(path.is_some());
    }

    #[test]
    #[ignore]
    fn test_find_correspondence() {
        let path = find_corresponding_path(&current_dir().unwrap(), ".s3-prefix");
        eprintln!("{:?}", path);
    }
}
