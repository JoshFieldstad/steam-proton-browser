//! Open files and folders via the OS or a terminal editor.

use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};

/// Open the given directory in the system's default file manager.
pub fn open_in_file_explorer(path: &Path) -> Result<()> {
    open::that_detached(path)?;
    Ok(())
}

/// Open a file with the OS default application (detached).
pub fn open_file(path: &Path) -> Result<()> {
    open::that_detached(path)?;
    Ok(())
}

/// Open a file in the user's preferred terminal editor.
/// Checks `$EDITOR`, then `$VISUAL`, then falls back to common defaults.
/// This is a blocking call — the TUI should suspend while the editor runs.
pub fn open_in_editor(path: &Path) -> Result<()> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| default_editor().to_string());

    let status = Command::new(&editor).arg(path).status()?;

    if !status.success() {
        bail!("{editor} exited with {status}");
    }

    Ok(())
}

fn default_editor() -> &'static str {
    "vi"
}
