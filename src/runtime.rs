//! Storage selection for installed and portable distributions.
use std::path::{Path, PathBuf};

fn portable_storage(executable: &Path) -> Option<PathBuf> {
    let directory = executable.parent()?;
    directory
        .join("portable.txt")
        .is_file()
        .then(|| directory.join("data").join("state.ron"))
}

pub fn persistence_path() -> std::io::Result<Option<PathBuf>> {
    let path = portable_storage(&std::env::current_exe()?);
    if let Some(path) = &path {
        std::fs::create_dir_all(path.parent().expect("portable storage has a parent"))?;
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_storage_follows_executable_and_requires_marker() {
        let directory =
            std::env::temp_dir().join(format!("smithcraft-portable-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let executable = directory.join("SmithCraft.exe");
        assert_eq!(portable_storage(&executable), None);
        std::fs::write(directory.join("portable.txt"), "").unwrap();
        assert_eq!(
            portable_storage(&executable),
            Some(directory.join("data/state.ron"))
        );
        std::fs::remove_file(directory.join("portable.txt")).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
