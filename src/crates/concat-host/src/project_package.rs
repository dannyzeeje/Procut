// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer Abdullah and Concat contributors
//! Portable project packages: manifests, referenced media and fonts.
use crate::projects;
use serde_json::Value;
use std::io::{Read, Write};
use std::path::{Component, Path};
use zip::{ZipArchive, ZipWriter, write::FileOptions};
const LIMIT: u64 = 16 * 1024 * 1024 * 1024;
/// Export a copy of the manifest and referenced files without modifying the project.
pub fn export(document: &Value, project: &Path, output: &Path) -> Result<(), String> {
    export_with_looks(document, project, output, None)
}
/// Export with imported LUT tables, regenerating their shaders on import.
pub fn export_with_looks(
    document: &Value,
    project: &Path,
    output: &Path,
    looks: Option<&Path>,
) -> Result<(), String> {
    let mut doc = document.clone();
    let temp = output.with_extension("procut.tmp");
    let result = (|| {
        let mut z = ZipWriter::new(std::fs::File::create(&temp).map_err(|e| e.to_string())?);
        let opt = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        let mut total = 0u64;
        for section in ["media", "fonts"] {
            if let Some(items) = doc.get_mut(section).and_then(Value::as_array_mut) {
                for (i, item) in items.iter_mut().enumerate() {
                    let Some(raw) = item.get("path").and_then(Value::as_str) else {
                        continue;
                    };
                    let path = if Path::new(raw).is_absolute() {
                        Path::new(raw).to_path_buf()
                    } else {
                        project.join(raw)
                    };
                    total = total
                        .checked_add(
                            path.metadata()
                                .map_err(|e| format!("{}: {e}", path.display()))?
                                .len(),
                        )
                        .ok_or("Package too large")?;
                    if total > LIMIT {
                        return Err("Project package exceeds 16 GiB".into());
                    }
                    let name = format!(
                        "assets/{section}-{i}.{}",
                        path.extension().and_then(|e| e.to_str()).unwrap_or("bin")
                    );
                    z.start_file(&name, opt).map_err(|e| e.to_string())?;
                    std::io::copy(
                        &mut std::fs::File::open(path).map_err(|e| e.to_string())?,
                        &mut z,
                    )
                    .map_err(|e| e.to_string())?;
                    item["path"] = name.into();
                }
            }
        }
        if let Some(looks) = looks {
            let mut ids = std::collections::HashSet::new();
            collect_looks(&doc, &mut ids);
            for id in ids {
                if !safe_id(&id) {
                    return Err("Invalid LUT id".into());
                }
                let table = std::fs::read(looks.join(&id).join(concat_effects::looks::TABLE))
                    .map_err(|e| format!("Cannot package custom look {id}: {e}"))?;
                if table.len() > 32 * 1024 * 1024 {
                    return Err("LUT table exceeds 32 MiB".into());
                }
                z.start_file(format!("assets/luts/{id}.cube"), opt)
                    .map_err(|e| e.to_string())?;
                z.write_all(&table).map_err(|e| e.to_string())?;
            }
        }
        z.start_file("concat.json", opt)
            .map_err(|e| e.to_string())?;
        z.write_all(&serde_json::to_vec_pretty(&doc).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        z.finish()
            .map_err(|e| e.to_string())?
            .sync_all()
            .map_err(|e| e.to_string())?;
        std::fs::rename(&temp, output).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temp);
    }
    result
}
/// Extract a validated archive into a new project folder, with traversal and expansion limits.
pub fn import(package: &Path, location: &Path) -> Result<projects::ProjectInfo, String> {
    let mut z = ZipArchive::new(std::fs::File::open(package).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if z.len() > 20000 {
        return Err("Too many package entries".into());
    }
    let mut names = std::collections::HashSet::new();
    let mut total = 0u64;
    for i in 0..z.len() {
        let f = z.by_index(i).map_err(|e| e.to_string())?;
        let p = f.enclosed_name().ok_or("Unsafe archive path")?;
        if p.components().any(|c| !matches!(c, Component::Normal(_)))
            || f.name().contains('\\')
            || (p != Path::new("concat.json") && !p.starts_with("assets"))
        {
            return Err("Unsafe archive entry".into());
        }
        if f.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000)
            || !names.insert(f.name().to_owned())
        {
            return Err("Symlink or duplicate archive entry".into());
        }
        total = total.checked_add(f.size()).ok_or("Package too large")?;
        if total > LIMIT {
            return Err("Package exceeds 16 GiB".into());
        }
    }
    let mut raw = String::new();
    z.by_name("concat.json")
        .map_err(|e| e.to_string())?
        .take(16 * 1024 * 1024)
        .read_to_string(&mut raw)
        .map_err(|e| e.to_string())?;
    let mut doc: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    std::fs::create_dir_all(location).map_err(|e| e.to_string())?;
    let root = location.join(format!("Imported-{stamp}"));
    std::fs::create_dir(&root).map_err(|e| e.to_string())?;
    let result = (|| {
        for i in 0..z.len() {
            let mut file = z.by_index(i).map_err(|e| e.to_string())?;
            let relative = file.enclosed_name().ok_or("Unsafe path")?.to_path_buf();
            let dest = root.join(relative);
            if file.is_dir() {
                std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
                continue;
            }
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let expected = file.size();
            let mut output = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(dest)
                .map_err(|e| e.to_string())?;
            let written = std::io::copy(
                &mut (&mut file).take(expected.saturating_add(1)),
                &mut output,
            )
            .map_err(|e| e.to_string())?;
            if written != expected {
                return Err("Archive entry size mismatch".into());
            }
        }
        for section in ["media", "fonts"] {
            if let Some(items) = doc.get_mut(section).and_then(Value::as_array_mut) {
                for item in items {
                    let Some(raw) = item.get("path").and_then(Value::as_str) else {
                        continue;
                    };
                    let p = Path::new(raw);
                    if !p.starts_with("assets")
                        || p.components().any(|c| !matches!(c, Component::Normal(_)))
                    {
                        return Err("A project reference leaves its package".into());
                    }
                    let file = root.join(p);
                    if !file.is_file() {
                        return Err("Referenced package file is missing".into());
                    }
                    item["path"] = file.to_string_lossy().into_owned().into();
                }
            }
        }
        projects::save(&root.to_string_lossy(), &doc)?;
        projects::open(&root.to_string_lossy())
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&root);
    }
    result
}

fn safe_id(id: &str) -> bool {
    id.starts_with("user.")
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".-_".contains(&c))
}
fn collect_looks(value: &Value, ids: &mut std::collections::HashSet<String>) {
    match value {
        Value::Array(a) => {
            for v in a {
                collect_looks(v, ids);
            }
        }
        Value::Object(o) => {
            if let Some(id) = o
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| id.starts_with("user."))
            {
                ids.insert(id.to_owned());
            }
            for v in o.values() {
                collect_looks(v, ids);
            }
        }
        _ => {}
    }
}
fn remap(value: &mut Value, old: &str, new: &str) {
    match value {
        Value::Array(a) => {
            for v in a {
                remap(v, old, new);
            }
        }
        Value::Object(o) => {
            if o.get("id").and_then(Value::as_str) == Some(old) {
                o.insert("id".into(), Value::String(new.into()));
            }
            for v in o.values_mut() {
                remap(v, old, new);
            }
        }
        _ => {}
    }
}
/// Import LUT tables under unique ids, keeping existing projects' looks unchanged.
pub fn import_with_looks(
    package: &Path,
    location: &Path,
    looks: &Path,
) -> Result<projects::ProjectInfo, String> {
    let info = import(package, location)?;
    let root = Path::new(&info.path);
    let result = (|| {
        let mut doc: Value = serde_json::from_slice(
            &std::fs::read(projects::manifest_path(root)).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if let Ok(tables) = std::fs::read_dir(root.join("assets/luts")) {
            for table in tables {
                let table = table.map_err(|e| e.to_string())?.path();
                if table.metadata().map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
                    return Err("LUT exceeds 32 MiB".into());
                }
                let old = table
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or("Invalid LUT filename")?;
                if !safe_id(old) {
                    return Err("Invalid imported LUT id".into());
                }
                let unique = root.join("assets").join(format!(
                    "{}-{old}.cube",
                    root.file_name().unwrap_or_default().to_string_lossy()
                ));
                std::fs::copy(&table, &unique).map_err(|e| e.to_string())?;
                let new = concat_effects::looks::import(looks, &unique)?;
                remap(&mut doc, old, &new);
            }
        }
        projects::save(&info.path, &doc)?;
        Ok(info.clone())
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(root);
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_media_roundtrip() {
        let d = tempfile::tempdir().unwrap();
        let media = d.path().join("source.mp4");
        std::fs::write(&media, b"media").unwrap();
        let info = projects::create(
            &d.path().to_string_lossy(),
            "Test",
            640,
            480,
            30,
            1,
            [0, 0, 0, 255],
        )
        .unwrap();
        let mut doc: Value = serde_json::from_slice(
            &std::fs::read(projects::manifest_path(Path::new(&info.path))).unwrap(),
        )
        .unwrap();
        doc["media"] = serde_json::json!([{"path":media.to_string_lossy()}]);
        let out = d.path().join("test.procut");
        export(&doc, Path::new(&info.path), &out).unwrap();
        let restored = import(&out, &d.path().join("imports")).unwrap();
        let value: Value = serde_json::from_slice(
            &std::fs::read(projects::manifest_path(Path::new(&restored.path))).unwrap(),
        )
        .unwrap();
        assert_eq!(
            std::fs::read(value["media"][0]["path"].as_str().unwrap()).unwrap(),
            b"media"
        );
        assert!(media.exists());
    }
    #[test]
    fn custom_look_roundtrip_preserves_names_and_remaps_only_ids() {
        let d = tempfile::tempdir().unwrap();
        let info = projects::create(
            &d.path().to_string_lossy(),
            "user.original",
            640,
            480,
            30,
            1,
            [0, 0, 0, 255],
        )
        .unwrap();
        let looks = d.path().join("looks");
        let cube = d.path().join("original.cube");
        let mut table = String::from("LUT_3D_SIZE 2\n");
        for b in 0..2 {
            for g in 0..2 {
                for r in 0..2 {
                    table.push_str(&format!("{r} {g} {b}\n"));
                }
            }
        }
        std::fs::write(&cube, table).unwrap();
        let old = concat_effects::looks::import(&looks, &cube).unwrap();
        let mut doc: Value = serde_json::from_slice(
            &std::fs::read(projects::manifest_path(Path::new(&info.path))).unwrap(),
        )
        .unwrap();
        doc["name"] = old.clone().into();
        doc["test_link"] = serde_json::json!({"id":old});
        let package = d.path().join("custom.procut");
        export_with_looks(&doc, Path::new(&info.path), &package, Some(&looks)).unwrap();
        let restored = import_with_looks(&package, &d.path().join("imports"), &looks).unwrap();
        let doc: Value = serde_json::from_slice(
            &std::fs::read(projects::manifest_path(Path::new(&restored.path))).unwrap(),
        )
        .unwrap();
        assert_eq!(doc["name"], old);
        let new = doc["test_link"]["id"].as_str().unwrap();
        assert_ne!(new, old);
        assert!(looks.join(new).join(concat_effects::looks::TABLE).exists());
    }
    #[test]
    fn rejects_zip_traversal() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("bad.procut");
        let mut z = ZipWriter::new(std::fs::File::create(&p).unwrap());
        z.start_file("../escaped", FileOptions::default()).unwrap();
        z.write_all(b"bad").unwrap();
        z.finish().unwrap();
        assert!(import(&p, &d.path().join("imports")).is_err());
        assert!(!d.path().join("escaped").exists());
    }
}
