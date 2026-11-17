// SPDX-License-Identifier: AGPL-3.0-or-later
//! Temporary effect edits, rendered on the echo and committed only on Apply.
use super::*;

pub(super) struct EffectPreview {
    commands: Vec<Command>,
    selection: Vec<String>,
    playhead: f32,
    playing: bool,
    start: f32,
    end: f32,
}

impl Studio {
    /// Begin a temporary effect edit without changing session history.
    pub fn preview_effect(&mut self, kind: &str, id: &str, label: &str) {
        self.finish_effect_preview(false);
        self.flush_commit();
        let clip = self.sole_selection().and_then(|id| self.clip(&id));
        let start = clip.map_or(self.playhead, |clip| clip.start as f32);
        let end = clip.map_or((start + 3.0).min(self.duration()), |clip| {
            (clip.start + clip.duration) as f32
        });
        let preview = EffectPreview {
            commands: Vec::new(),
            selection: self.selection.clone(),
            playhead: self.playhead,
            playing: self.playing,
            start,
            end,
        };
        self.pause();
        self.flat = None;
        self.begin_echo();
        self.effect_preview = Some(preview);
        match kind {
            "filter" => {
                self.place_filter_layer(id, label);
            }
            "transition" => {
                self.apply_transition(id);
            }
            "effect" => {
                self.apply_catalogue(id, true);
            }
            "animation" => {}
            _ => {
                self.finish_effect_preview(false);
                return;
            }
        }
        // A new filter or transition owns its own span; an effect or
        // animation uses the source clip's span.
        let (start, end) = self
            .sole_selection()
            .and_then(|id| self.clip(&id))
            .map_or((start, end), |clip| {
                (clip.start as f32, (clip.start + clip.duration) as f32)
            });
        if let Some(preview) = &mut self.effect_preview {
            preview.start = start;
            preview.end = end;
        }
        self.seek(start);
        if end > start {
            self.play_toggle();
        }
    }

    pub(super) fn preview_command(&mut self, command: Command) -> Option<String> {
        self.flat = None;
        let echo = self.echo.as_mut()?;
        let mut mint = concat_project::commands::IdMint::default();
        mint.adopt_project(echo);
        match concat_project::commands::apply(echo, &mut mint, command.clone()) {
            Ok(outcome) => {
                self.effect_preview.as_mut()?.commands.push(command);
                outcome.created_id
            }
            Err(error) => {
                self.notify(&error.to_string(), true);
                None
            }
        }
    }

    /// Commit the preview once, or discard it and restore playback state.
    pub fn finish_effect_preview(&mut self, accept: bool) {
        let Some(preview) = self.effect_preview.take() else {
            return;
        };
        self.pause();
        self.flat = None;
        self.selection = preview.selection;
        if accept {
            if preview.commands.is_empty() {
                self.commit_now();
            } else {
                self.echo = None;
                self.apply(Command::Batch {
                    commands: preview.commands,
                });
            }
        } else {
            self.echo = None;
            self.commit_targets.clear();
        }
        self.flat = None;
        self.seek(preview.playhead);
        if preview.playing {
            self.play_toggle();
        }
        self.request_preview();
    }

    pub(super) fn preview_loop_span(&self) -> Option<(f32, f32)> {
        self.effect_preview
            .as_ref()
            .map(|preview| (preview.start, preview.end))
    }
}
