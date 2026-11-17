// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer Abdullah and Concat contributors
//! Recoverable project-library removal. Only the project folder moves; external media stays put.
use crate::projects;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize, Deserialize)]
/// A project held in the reversible trash.
pub struct Entry {
    /// Original project location.
    pub original: String,
    /// Folder holding the project while trashed.
    pub stored: String,
}
fn index(config: &Path) -> PathBuf {
    config.join("project-trash.json")
}
/// Read the trash index, refusing corrupt data rather than overwriting it.
pub fn entries(config: &Path) -> Result<Vec<Entry>, String> {
    match std::fs::read(index(config)) {
        Ok(data) => serde_json::from_slice(&data).map_err(|e| e.to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(vec![]),
        Err(e) => Err(e.to_string()),
    }
}
fn write(config: &Path, data: &[Entry]) -> Result<(), String> {
    std::fs::create_dir_all(config).map_err(|e| e.to_string())?;
    let temp = config.join("project-trash.json.tmp");
    std::fs::write(
        &temp,
        serde_json::to_vec_pretty(data).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(temp, index(config)).map_err(|e| e.to_string())
}
/// Move a project into a sibling trash folder, preserving filesystem ownership and media paths for restore.
pub fn trash(config: &Path, path: &str) -> Result<(), String> {
    let source = Path::new(path).canonicalize().map_err(|e| e.to_string())?;
    if !projects::is_project(&source) {
        return Err("Not a project folder".into());
    }
    let mut list = entries(config)?;
    let parent = source
        .parent()
        .ok_or("No parent folder")?
        .join(".procut-trash");
    std::fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let stored = parent.join(format!(
        "{stamp}-{}",
        source
            .file_name()
            .ok_or("No project name")?
            .to_string_lossy()
    ));
    let entry = Entry {
        original: source.to_string_lossy().into_owned(),
        stored: stored.to_string_lossy().into_owned(),
    };
    // Journal the destination first. A failed move is rolled back in the index.
    list.push(entry.clone());
    write(config, &list)?;
    if let Err(e) = std::fs::rename(&source, &stored) {
        list.pop();
        let _ = write(config, &list);
        return Err(e.to_string());
    }
    if let Err(e) = projects::forget(config, path) {
        let _ = std::fs::rename(&stored, &source);
        list.pop();
        let _ = write(config, &list);
        return Err(e);
    }
    Ok(())
}
/// Restore without replacing an existing project.
pub fn restore(config: &Path, stored: &str) -> Result<(), String> {
    let mut list = entries(config)?;
    let at = list
        .iter()
        .position(|e| e.stored == stored)
        .ok_or("Project is not in trash")?;
    let item = list[at].clone();
    if Path::new(&item.original).exists() {
        return Err("The original folder already exists; move it before restoring".into());
    }
    std::fs::rename(&item.stored, &item.original).map_err(|e| e.to_string())?;
    let info = projects::open(&item.original)?;
    projects::remember(config, &info)?;
    list.remove(at);
    write(config, &list)
}
/// Rediscover projects removed from recents under known project roots.
pub fn recover(config: &Path, roots: &[PathBuf]) -> Result<usize, String> {
    let mut count = 0;
    for root in roots {
        let Ok(children) = std::fs::read_dir(root) else {
            continue;
        };
        for child in children.flatten() {
            let p = child.path();
            if !p.is_dir()
                || p.file_name()
                    .is_some_and(|s| s.to_string_lossy().starts_with('.'))
            {
                continue;
            }
            if let Ok(info) = projects::open(&p.to_string_lossy()) {
                projects::remember(config, &info)?;
                count += 1;
            }
        }
    }
    Ok(count)
}

/// Permanently remove a journaled trash folder, never an arbitrary path.
pub fn erase(config: &Path, stored: &str) -> Result<(), String> {
    let mut list = entries(config)?;
    let index = list
        .iter()
        .position(|e| e.stored == stored)
        .ok_or("Not a trashed project")?;
    let folder = Path::new(stored)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if folder
        .parent()
        .and_then(Path::file_name)
        .is_none_or(|s| s != ".procut-trash")
    {
        return Err("Invalid trash location".into());
    }
    if !projects::is_project(&folder) {
        return Err("Trash entry is not a project".into());
    }
    std::fs::remove_dir_all(folder).map_err(|e| e.to_string())?;
    list.remove(index);
    write(config, &list)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trash_restore_and_recovery_preserve_project_files() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("config");
        let home = root.path().join("projects");
        let info = projects::create(
            &home.to_string_lossy(),
            "Edit",
            1280,
            720,
            30,
            1,
            [0, 0, 0, 255],
        )
        .unwrap();
        projects::remember(&config, &info).unwrap();
        projects::forget(&config, &info.path).unwrap();
        assert_eq!(recover(&config, &[home]), Ok(1));
        trash(&config, &info.path).unwrap();
        assert!(!Path::new(&info.path).exists());
        let list = entries(&config).unwrap();
        assert_eq!(list.len(), 1);
        restore(&config, &list[0].stored).unwrap();
        assert!(projects::is_project(Path::new(&info.path)));
        assert!(entries(&config).unwrap().is_empty());
    }
}
