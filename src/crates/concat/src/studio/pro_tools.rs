// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

pub(super) use concat_effects::catalogue::CAMERA_INPUT_ID;
const LOW_PASS_ID: &str = "concat.low-pass";
const HIGH_PASS_ID: &str = "concat.high-pass";

pub(super) fn is_grade_layer(entry: &AppliedFilter) -> bool {
    entry.id == ADJUST_ID
        || matches!(entry.id.as_str(), "concat.color-wheels" | "concat.curves")
        || Catalogue::builtin()
            .get(&entry.id)
            .is_some_and(|package| package.lut().is_some())
}

impl Studio {
    pub fn phone_workspace_changed(&mut self, mode: i32) {
        self.phone_color_workspace = mode == 1;
        let audio = mode == 2;
        if !audio {
            self.phone_audio_loop = None;
        }
        self.flush_commit();
        self.phone_color_compare(false);
        if audio != self.meters.phone_workspace {
            let zoom = self.lanes.seconds_per_pixel;
            let next = if audio {
                self.phone_edit_zoom = Some(zoom);
                self.phone_audio_zoom
            } else {
                self.phone_audio_zoom = Some(zoom);
                self.phone_edit_zoom
            };
            if let Some(next) = next {
                self.lanes.seconds_per_pixel = next;
                self.lanes.recentre(self.playhead);
            }
            self.meters.phone_workspace = audio;
        }
        self.wake_meters();
    }

    pub(super) fn publish_phone_grade(&self, editor: &Editor, models: &Models) {
        let chain = self
            .phone_grade_clip()
            .map_or(&[][..], |clip| clip.video_effects.as_slice());
        let catalogue = Catalogue::builtin();
        let base = chain
            .iter()
            .find(|entry| entry.id == CAMERA_INPUT_ID && entry.enabled);
        let setting = |key| {
            base.and_then(|entry| entry.params.get(key))
                .copied()
                .unwrap_or(0.0) as i32
        };
        editor.set_phone_camera_profile(setting("profile"));
        editor.set_phone_camera_levels(setting("levels"));

        let indexed: Vec<_> = chain
            .iter()
            .enumerate()
            .filter(|(_, entry)| is_grade_layer(entry))
            .collect();
        let at = self.key_point().map(|(_, at)| at);
        let (_, mut params) = chain_rows(chain);
        let mut layers = Vec::with_capacity(indexed.len());
        for (position, &(index, entry)) in indexed.iter().enumerate() {
            let name = catalogue
                .get(&entry.id)
                .map(|package| package.manifest.effect.name.clone())
                .unwrap_or_else(|| label_of(&entry.id));
            layers.push(crate::ui::GradeLayerData {
                index: index as i32,
                name: format!("{} · {name}", position + 1).into(),
                primaries: entry.id == ADJUST_ID,
                wheels: entry.id == ADJUST_ID || entry.id == "concat.color-wheels",
                curves: entry.id == ADJUST_ID || entry.id == "concat.curves",
                on: entry.enabled,
                up: position > 0,
                down: position + 1 < indexed.len(),
            });
            if entry.id == ADJUST_ID {
                params.extend(
                    adjust_rows(std::slice::from_ref(entry), at)
                        .into_iter()
                        .map(|mut param| {
                            param.entry = index as i32;
                            param.keyable = at.is_some();
                            param
                        }),
                );
            }
        }
        sync(&models.phone_grade_layers, layers);
        sync(&models.phone_grade_params, params);
    }

    pub fn phone_camera_input(&mut self, profile: i32, levels: i32) {
        if !(0..=5).contains(&profile) || !(0..=1).contains(&levels) {
            return;
        }
        self.flush_commit();
        let Some(clip) = self.phone_grade_clip() else {
            return;
        };
        let clip_id = clip.id.clone();
        let mut layers = clip.video_effects.clone();
        layers.retain(|entry| entry.id != CAMERA_INPUT_ID);
        if profile > 0 {
            let mut input = AppliedFilter::new(CAMERA_INPUT_ID);
            input
                .params
                .insert("profile".to_owned(), f64::from(profile));
            input.params.insert("levels".to_owned(), f64::from(levels));
            layers.insert(0, input);
        }
        if clip.video_effects != layers {
            self.apply(Command::UpdateClip {
                clip_id,
                patch: ClipPatch {
                    video_effects: Some(layers),
                    ..Default::default()
                },
            });
        }
    }

    pub fn phone_grade_layer(&mut self, action: &str, index: i32) {
        self.flush_commit();
        let Some(clip) = self.phone_grade_clip() else {
            return;
        };
        if action == "add" {
            self.chain_add(false, ADJUST_ID);
            return;
        }
        let Some(index) = usize::try_from(index)
            .ok()
            .filter(|&index| clip.video_effects.get(index).is_some_and(is_grade_layer))
        else {
            return;
        };
        match action {
            "toggle" => self.chain_toggle(false, index as i32),
            "remove" => self.chain_remove(false, index as i32),
            "up" | "down" => {
                let layers = &clip.video_effects;
                let peer = if action == "up" {
                    layers[..index].iter().rposition(is_grade_layer)
                } else {
                    layers[index + 1..]
                        .iter()
                        .position(is_grade_layer)
                        .map(|i| index + 1 + i)
                };
                if let Some(peer) = peer {
                    self.chain_move(false, index as i32, peer as i32 - index as i32);
                }
            }
            _ => {}
        }
    }

    pub fn phone_track_mix(&mut self, id: &str, field: &str, value: f32) {
        if !value.is_finite() || self.locked(id) {
            return;
        }
        let Some(track) = self.timeline().tracks.iter().find(|track| track.id == id) else {
            return;
        };
        let mut mix = track.audio_mix();
        match field {
            "gain" => mix.gain_db = f64::from(value),
            "pan" => mix.pan = f64::from(value),
            "solo" => mix.solo = value != 0.0,
            "mute" => {
                self.apply(Command::SetTrackFlag {
                    track_id: id.to_owned(),
                    flag: TrackFlag::Muted,
                    value: value != 0.0,
                });
                return;
            }
            _ => return,
        }
        if mix.bounded() != track.audio_mix() {
            self.apply(Command::SetTrackMix {
                track_id: id.to_owned(),
                mix,
            });
        }
    }

    pub fn phone_audio_filter(&mut self, high_pass: bool) {
        let Some(clip) = self.sole_selection().and_then(|id| self.clip(&id)) else {
            return;
        };
        if !self.locked(&clip.track_id)
            && (self.clip_has_sound(clip) || clip.kind == model::ClipKind::SoundLayer)
        {
            self.chain_add(true, if high_pass { HIGH_PASS_ID } else { LOW_PASS_ID });
        }
    }

    pub fn phone_trim_edge(&mut self, at_start: bool, frames: i32) {
        if frames.abs_diff(0) != 1 {
            return;
        }
        let Some(id) = self.sole_selection() else {
            return;
        };
        let Some(clip) = self.clip(&id) else { return };
        if self.locked(&clip.track_id) {
            return;
        }
        let frame = 1.0 / self.timeline().video.rate().max(1.0);
        let delta = f64::from(frames) * frame;
        if clip.duration + if at_start { -delta } else { delta } < frame - 1e-9 {
            return;
        }
        self.apply(Command::TrimClip {
            clip_id: id.clone(),
            edge: if at_start {
                TrimEdge::Start
            } else {
                TrimEdge::End
            },
            delta,
            ripple: self.prefs.magnetic(),
        });
        if let Some(clip) = self.clip(&id) {
            let time = if at_start {
                clip.start
            } else {
                (clip.start + clip.duration - frame).max(clip.start)
            };
            self.seek(time as f32);
        }
    }

    pub fn phone_color_loop(&mut self, enabled: bool) {
        self.phone_color_loop = enabled;
    }

    pub fn phone_audio_loop(&mut self, start: f32, end: f32) {
        self.phone_audio_loop = self
            .meters
            .phone_workspace
            .then(|| self.phone_audio_span(start, end))
            .flatten()
            .map(|(start, end)| (start as f32, end as f32));
    }

    fn phone_audio_span(&self, start: f32, end: f32) -> Option<(f64, f64)> {
        if self.session.is_none() || !start.is_finite() || !end.is_finite() {
            return None;
        }
        let fps = self.timeline().video.rate().max(1.0);
        let limit = f64::from(self.duration());
        let snap = |time: f32| ((f64::from(time) * fps).round() / fps).clamp(0.0, limit);
        let (start, end) = (snap(start), snap(end));
        (end - start + 1e-9 >= 1.0 / fps).then_some((start, end))
    }

    pub fn phone_filter_region(&mut self, start: f32, end: f32, high_pass: bool) {
        let Some((start, end)) = self.phone_audio_span(start, end) else {
            return;
        };
        let (filter, name) = if high_pass {
            (HIGH_PASS_ID, t("mobile.highPass"))
        } else {
            (LOW_PASS_ID, t("mobile.lowPass"))
        };
        if let Some(id) = self.apply(Command::AddLayerClip {
            track_id: None,
            start,
            duration: Some(end - start),
            effect_id: filter.to_owned(),
            name,
            sound: true,
        }) {
            self.selection = vec![id];
            self.transition_selected = None;
        }
    }

    pub fn phone_audio_key(&mut self, from: f32, at: f32, fraction: f32) {
        if !from.is_finite() || !at.is_finite() || !fraction.is_finite() {
            return;
        }
        let Some(id) = self.sole_selection() else {
            return;
        };
        let Some(clip) = self.clip(&id) else { return };
        if self.locked(&clip.track_id) || !self.clip_has_sound(clip) {
            return;
        }
        let at = at.clamp(0.0, 1.0);
        if from >= 0.0 {
            self.drag_key(ClipField::Volume, "", from, at, fraction.clamp(0.0, 1.0));
            return;
        }
        let property = model::KeyProperty::Volume;
        let (min, max) = knob_range(property);
        let value = model_value(
            property,
            min + (max - min) * f64::from(fraction.clamp(0.0, 1.0)),
        );
        let ease = clip
            .key_at(property, f64::from(at))
            .map_or(model::KeyEase::LINEAR, |index| clip.keys[index].ease);
        self.apply(Command::SetClipKey {
            clip_id: id,
            property,
            at: f64::from(at),
            value,
            ease,
        });
    }

    pub fn phone_audio_remove_key(&mut self, at: f32) {
        let Some(id) = self.sole_selection() else {
            return;
        };
        if at.is_finite()
            && self
                .clip(&id)
                .is_some_and(|clip| self.clip_has_sound(clip) && !self.locked(&clip.track_id))
        {
            self.apply(Command::ClearClipKey {
                clip_id: id,
                property: model::KeyProperty::Volume,
                at: f64::from(at),
            });
        }
    }
}
