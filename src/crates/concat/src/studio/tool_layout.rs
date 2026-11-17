// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-section tool order, visibility and quick-bar placement.
use super::*;
use crate::ui::{Glyph, MobileToolData};
use slint::Model;

impl Studio {
    fn mobile_tool_section(&self) -> &'static str {
        match self
            .selection
            .first()
            .and_then(|id| self.clip(id))
            .map(|clip| clip.kind)
        {
            Some(model::ClipKind::Audio | model::ClipKind::SoundLayer) => "audio",
            Some(model::ClipKind::Video | model::ClipKind::Image) => "video",
            Some(model::ClipKind::Text) => "text",
            Some(model::ClipKind::Shape) => "shape",
            Some(_) => "layer",
            None => "project",
        }
    }
    fn mobile_tool_data(&self) -> Vec<MobileToolData> {
        let section = self.mobile_tool_section();
        let clip = section != "project";
        let picture = section == "video";
        let av = picture || section == "audio";
        let visual = picture || section == "text" || section == "shape" || section == "layer";
        let layer = section == "layer";
        let shape = section == "shape";
        let mut tools = Vec::new();
        if !clip {
            tools.push(("media", "common.media", Glyph::Film, true));
        }
        if !clip {
            tools.push(("audio", "common.audio", Glyph::Music, true));
        }
        if !clip {
            tools.push(("text", "common.text", Glyph::TextMark, true));
        }
        if !clip {
            tools.push(("effects", "common.effects", Glyph::Sparkle, false));
        }
        if !clip {
            tools.push(("filters", "mediaPane.filters", Glyph::Filter, false));
        }
        if !clip {
            tools.push((
                "adjustments",
                "mediaPane.adjustments",
                Glyph::Sliders,
                false,
            ));
        }
        if !clip {
            tools.push((
                "transitions",
                "mediaPane.transitions",
                Glyph::Transition,
                false,
            ));
        }
        if !clip {
            tools.push(("captions", "phone.captions", Glyph::TextMark, false));
        }
        if !clip {
            tools.push(("speak", "phone.speak", Glyph::Volume, false));
        }
        if !clip {
            tools.push(("voiceover", "phone.voiceover", Glyph::Mic, false));
        }
        if !clip {
            tools.push(("canvas", "phone.canvas", Glyph::Frame, false));
        }
        if !clip {
            tools.push(("project", "phone.project", Glyph::Settings, false));
        }
        if clip {
            tools.push(("split", "phone.split", Glyph::Split, true));
        }
        if clip {
            tools.push(("delete", "common.delete", Glyph::Trash, true));
        }
        if clip {
            tools.push(("edit", "phone.edit", Glyph::Frame, true));
        }
        if picture {
            tools.push(("clip-canvas", "phone.canvas", Glyph::Frame, false));
        }
        if visual {
            tools.push(("clip-adjust", "common.adjust", Glyph::Sliders, false));
        }
        if picture {
            tools.push(("clip-enhance", "common.enhance", Glyph::Sparkle, false));
        }
        if visual {
            tools.push(("clip-effects", "common.effects", Glyph::Sparkle, false));
        }
        if av {
            tools.push(("clip-speed", "common.speed", Glyph::Clock, false));
        }
        if av {
            tools.push(("clip-audio", "common.audio", Glyph::Volume, false));
        }
        if av {
            tools.push((
                "clip-audio-effects",
                "configPane.audioEffects",
                Glyph::Waveform,
                false,
            ));
        }
        if visual {
            tools.push(("clip-animate", "motion.animate", Glyph::Play, false));
        }
        if picture {
            tools.push(("clip-mask", "mask.title", Glyph::MaskCircle, false));
        }
        if picture {
            tools.push((
                "transitions",
                "mediaPane.transitions",
                Glyph::Transition,
                false,
            ));
        }
        if av || picture {
            tools.push(("replace", "mobile.replaceClip", Glyph::Import, false));
        }
        if clip {
            tools.push(("duplicate", "phone.duplicate", Glyph::Copy, false));
        }
        if picture {
            tools.push(("freeze", "timelinePane.freeze", Glyph::Image, false));
        }
        if av {
            tools.push(("reverse", "common.reverse", Glyph::StepBack, false));
        }
        if picture {
            tools.push(("mirror", "timelinePane.mirror", Glyph::FlipH, false));
        }
        if picture {
            tools.push(("rotate", "timelinePane.rotate", Glyph::Rotate, false));
        }
        if layer {
            tools.push(("clip-layer", "configPane.filter", Glyph::Filter, false));
        }
        if shape {
            tools.push(("clip-shape", "shapePanel.shape", Glyph::Shapes, false));
        }
        if let Some(order) = self.prefs.mobile_tool_order.get(section) {
            tools.sort_by_key(|tool| {
                order
                    .iter()
                    .position(|id| id == tool.0)
                    .unwrap_or(usize::MAX)
            });
        }
        tools
            .into_iter()
            .map(|(id, label, glyph, default_pin)| MobileToolData {
                id: id.into(),
                label: t(label).into(),
                glyph,
                pinned: self
                    .prefs
                    .mobile_tool_pinned
                    .get(section)
                    .map_or(default_pin, |ids| ids.iter().any(|item| item == id)),
                hidden: self
                    .prefs
                    .mobile_tool_hidden
                    .get(section)
                    .is_some_and(|ids| ids.iter().any(|item| item == id)),
            })
            .collect()
    }
    /// Update and persist one tool layout preference for this section.
    pub fn customize_mobile_tool(&mut self, id: &str, action: &str) {
        let section = self.mobile_tool_section().to_owned();
        let data = self.mobile_tool_data();
        let Some(index) = data.iter().position(|tool| tool.id == id) else {
            return;
        };
        match action {
            "up" | "down" => {
                let mut order: Vec<String> = data.iter().map(|tool| tool.id.to_string()).collect();
                let target = if action == "up" {
                    index.saturating_sub(1)
                } else {
                    (index + 1).min(order.len() - 1)
                };
                order.swap(index, target);
                self.prefs.mobile_tool_order.insert(section, order);
            }
            "pin" => {
                let ids = self
                    .prefs
                    .mobile_tool_pinned
                    .entry(section)
                    .or_insert_with(|| {
                        data.iter()
                            .filter(|tool| tool.pinned)
                            .map(|tool| tool.id.to_string())
                            .collect()
                    });
                if ids.iter().any(|item| item == id) {
                    ids.retain(|item| item != id);
                } else {
                    ids.push(id.to_owned());
                }
            }
            "hide" => {
                let ids = self.prefs.mobile_tool_hidden.entry(section).or_default();
                if ids.iter().any(|item| item == id) {
                    ids.retain(|item| item != id);
                } else {
                    ids.push(id.to_owned());
                }
            }
            _ => return,
        }
        self.prefs.save(&self.host.dirs);
    }
    pub(super) fn publish_mobile_tools(&self, app: &App) {
        let editor = app.global::<Editor>();
        let all = self.mobile_tool_data();
        let quick: Vec<_> = all
            .iter()
            .filter(|tool| tool.pinned && !tool.hidden)
            .cloned()
            .collect();
        let more: Vec<_> = all
            .iter()
            .filter(|tool| !tool.pinned && !tool.hidden)
            .cloned()
            .collect();
        if editor.get_mobile_all_tools().iter().collect::<Vec<_>>() != all {
            editor.set_mobile_all_tools(slint::ModelRc::new(VecModel::from(all)));
        }
        if editor.get_mobile_quick_tools().iter().collect::<Vec<_>>() != quick {
            editor.set_mobile_quick_tools(slint::ModelRc::new(VecModel::from(quick)));
        }
        if editor.get_mobile_more_tools().iter().collect::<Vec<_>>() != more {
            editor.set_mobile_more_tools(slint::ModelRc::new(VecModel::from(more)));
        }
    }
}
