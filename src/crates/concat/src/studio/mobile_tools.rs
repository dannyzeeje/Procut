// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer Abdullah and Concat contributors
//! Mobile project exchange and non-destructive replacement tools.
use super::*;
impl Studio {
    /// Pick replacement footage; keep the timeline position, length and looks.
    pub fn replace_selected_media() {
        crate::host::pick_files_then(t("mobile.replaceClip"), None, |studio, _, _, paths| {
            let Some(path) = paths.first() else { return };
            let Some(id) = studio.selection.first().cloned() else {
                return;
            };
            let Some(clip) = studio.clip(&id).cloned() else {
                return;
            };
            if studio.locked(&clip.track_id) {
                studio.notify(&t("mobile.lockedTrack"), true);
                return;
            }
            match media::probe(&path.to_string_lossy()) {
                Ok(summary) => {
                    let item = summary.to_new_media();
                    let compatible = matches!(
                        (clip.kind, item.kind),
                        (model::ClipKind::Video, model::MediaKind::Video)
                            | (model::ClipKind::Image, model::MediaKind::Image)
                            | (model::ClipKind::Audio, model::MediaKind::Audio)
                    );
                    if !compatible {
                        studio.notify(&t("mobile.sameMediaKind"), true);
                        return;
                    }
                    // Reject too-short replacements rather than inserting a black tail.
                    if item.kind != model::MediaKind::Image
                        && item.duration.is_none_or(|d| {
                            d + 0.001 < concat_project::speed::source_consumed(&clip, clip.duration)
                        })
                    {
                        studio.notify(&t("mobile.replacementTooShort"), true);
                        return;
                    }
                    studio.apply(Command::ReplaceClipMedia {
                        clip_id: id,
                        item,
                        source_start: Some(0.0),
                        enhanced: false,
                        offset: 0.0,
                    });
                    studio.request_preview();
                }
                Err(e) => studio.notify(&e.to_string(), true),
            }
        });
    }
    /// Pack the current committed document and hand it to the device's file storage.
    pub fn export_project_package(&mut self) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        if let Err(e) = session.save(None) {
            self.notify(&e, true);
            return;
        }
        let document = session.document();
        let project = std::path::PathBuf::from(session.path());
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let output = self.host.dirs.data.join(format!("Procut-{stamp}.procut"));
        let looks = Self::looks_dir(&self.host.dirs);
        self.notify(&t("mobile.exportingProject"), false);
        crate::host::spawn_in_project(
            move || {
                concat_host::project_package::export_with_looks(
                    &document,
                    &project,
                    &output,
                    Some(&looks),
                )?;
                Ok::<_, String>(output)
            },
            |studio, _, _, result| match result {
                Ok(path) => {
                    let text = path.to_string_lossy().into_owned();
                    let (tx, rx) = std::sync::mpsc::channel();
                    if crate::platform::publish_document(path, "application/zip", move |result| {
                        let _ = tx.send(result);
                    }) {
                        crate::host::spawn(
                            move || rx.recv().unwrap_or_else(|e| Err(e.to_string())),
                            |studio, _, _, result| match result {
                                Ok(_) => studio.notify(&t("mobile.packageExported"), false),
                                Err(e) => studio.notify(&e, true),
                            },
                        );
                    } else {
                        studio.notify(&text, false);
                    }
                }
                Err(e) => studio.notify(&e, true),
            },
        );
    }
    fn imported_lut_folder(&mut self, id: &str) -> Option<std::path::PathBuf> {
        if !id.starts_with("user.")
            || !id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b".-_".contains(&c))
            || !Catalogue::builtin()
                .get(id)
                .is_some_and(|package| package.lut().is_some())
        {
            self.notify(&t("mobile.importedLutsOnly"), true);
            return None;
        }
        Some(Self::looks_dir(&self.host.dirs).join(id))
    }

    pub fn categorize_lut(&mut self, id: &str, category: &str) {
        let Some(folder) = self.imported_lut_folder(id) else {
            return;
        };
        if let Err(error) = write_lut_category(&folder, category) {
            self.notify(&error.to_string(), true);
            return;
        }
        *self.shelf_stamp.borrow_mut() = None;
    }

    /// Remove a look from the picker while keeping old projects renderable.
    pub fn remove_imported_lut(&mut self, id: &str) {
        let Some(folder) = self.imported_lut_folder(id) else {
            return;
        };
        if let Err(error) = std::fs::write(folder.join("lut-hidden"), []) {
            self.notify(&error.to_string(), true);
            return;
        }
        *self.shelf_stamp.borrow_mut() = None;
    }
}

pub(super) fn write_lut_category(folder: &std::path::Path, category: &str) -> std::io::Result<()> {
    let category: String = category
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(240)
        .collect();
    let path = folder.join("lut-category");
    if category.is_empty() {
        match std::fs::remove_file(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            result => result,
        }
    } else {
        let temporary = path.with_extension("tmp");
        std::fs::write(&temporary, category)?;
        std::fs::rename(temporary, path)
    }
}
