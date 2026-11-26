// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use concat_host::preview::FrameSpec;

impl Studio {
    /// Extra frames for a precise cut or a paused split comparison. Keep at
    /// most one decode and the most recent pending request.
    pub(crate) fn request_phone_pair(&mut self, editor: &Editor, mode: &str, at_start: bool) {
        self.phone_pair_generation = self.phone_pair_generation.wrapping_add(1);
        editor.set_phone_frames_ready(false);
        self.phone_pair_pending = (!mode.is_empty()).then(|| (mode.to_owned(), at_start));
        if !self.phone_pair_busy {
            self.render_phone_pair(editor);
        }
    }

    fn render_phone_pair(&mut self, editor: &Editor) {
        let Some((mode, at_start)) = self.phone_pair_pending.take() else {
            return;
        };
        if self.session.is_none() || self.playing {
            return;
        }
        let color = mode == "color";
        let time = if color {
            f64::from(self.preview_time())
        } else {
            let Some(clip) = self.sole_selection().and_then(|id| self.clip(&id)) else {
                return;
            };
            if at_start {
                clip.start
            } else {
                clip.start + clip.duration
            }
        };
        let video = self.timeline().video.clone();
        let fps = video.rate().max(1.0);
        let tick = (time * fps - 1e-7).ceil().max(0.0);
        let spec = FrameSpec {
            time: if color { time } else { tick / fps },
            width: 2,
            height: 2,
            moving: false,
            proxy: false,
            color_space: video.color_space,
            canvas: video.canvas,
            nearest: false,
        };
        let (width, height) = self.output_size();
        let scale = (480.0 / f64::from(width.max(height))).min(1.0);
        let side = |n| ((f64::from(n) * scale).round() as u32).max(2) & !1;
        let spec = FrameSpec {
            width: side(width),
            height: side(height),
            ..spec
        };
        let current = self.phone_color_before.take();
        let after = self.preview_clips();
        let before = if color {
            self.phone_color_before = self
                .phone_grade_clip()
                .map(|clip| (self.timeline().id.clone(), clip.id.clone()));
            self.preview_clips()
        } else {
            after.clone()
        };
        self.phone_color_before = current;
        let (Some((before, before_settings)), Some((after, after_settings))) = (before, after)
        else {
            return;
        };
        let before_spec = if color {
            spec
        } else {
            FrameSpec {
                time: ((tick - 1.0) / fps).max(0.0),
                ..spec
            }
        };
        let monitor = self.host.monitor.clone();
        let generation = self.phone_pair_generation;
        self.phone_pair_busy = true;
        spawn_in_project(
            move || {
                let read = |clips, settings, request| {
                    monitor
                        .frame(clips, settings, request)
                        .ok()
                        .and_then(|bytes| {
                            concat_core::frame::Frame::from_rgba(spec.width, spec.height, bytes)
                        })
                };
                let a = if !color && tick == 0.0 {
                    None
                } else {
                    read(before, &before_settings, before_spec)
                };
                let b = read(after, &after_settings, spec);
                (a, b)
            },
            move |studio, app, _, (a, b)| {
                studio.phone_pair_busy = false;
                let editor = app.global::<Editor>();
                if studio.phone_pair_generation == generation {
                    editor.set_phone_frame_a(a.as_ref().map(image_of).unwrap_or_default());
                    editor.set_phone_frame_b(b.as_ref().map(image_of).unwrap_or_default());
                    editor.set_phone_frames_ready(b.is_some() && (!color || a.is_some()));
                }
                studio.render_phone_pair(&editor);
            },
        );
    }
}
