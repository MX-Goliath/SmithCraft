//! Independently authored help, displayed in the Windows HTML Help viewer.
#[cfg(windows)]
pub fn open(topic: usize) -> Result<(), String> {
    const BYTES: &[u8] = include_bytes!("../assets/SmithHelp.chm");
    let hash = BYTES.iter().fold(14695981039346656037u64, |h, b| {
        (h ^ *b as u64).wrapping_mul(1099511628211)
    });
    let path = std::env::temp_dir().join(format!("smithcraft-help-{hash:016x}.chm"));
    if std::fs::read(&path).ok().as_deref() != Some(BYTES) {
        std::fs::write(&path, BYTES).map_err(|e| e.to_string())?;
    }
    // The OS viewer owns the browser, navigation tree, search and print UI.
    // Microsoft's HTML Help SDK documents this ms-its command-line syntax.
    let target = format!(
        "ms-its:{}::/topic{}.htm",
        path.display(),
        topic.clamp(1, 62)
    );
    let viewer =
        std::path::PathBuf::from(std::env::var_os("SystemRoot").ok_or("SystemRoot is missing")?)
            .join("hh.exe");
    std::process::Command::new(viewer)
        .arg(target)
        .spawn()
        .map_err(|e| format!("Windows could not open the help viewer: {e}"))?;
    Ok(())
}
#[cfg(not(windows))]
pub fn open(_topic: usize) -> Result<(), String> {
    Err("The native help viewer requires Windows. The help source is help/SmithHelp.md.".into())
}
