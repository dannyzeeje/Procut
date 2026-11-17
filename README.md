# Procut

An Android-focused fork of [Concat](https://github.com/jub0t/Concat), originally developed by **[Jareer Abdullah (jub0t)](https://github.com/jub0t)** and the Concat contributors. Procut builds on their video editor and rendering engine. This project is an independent fork; upstream features and authorship belong to Concat.

## Changes in this fork

- Android phone layouts based on the Procut Design Atlas: compact Projects screen, reachable tool sheets, grouped tool buttons, readable timeline ruler and playhead, and a resizable preview/timeline split.
- Original Procut diamond logo supplied by the fork owner, recreated as SVG and Android vector artwork; purple default theme and startup animation.
- New phone projects start with V1 above A1. Video and audio tracks have separate numbering, with extra tracks created as content is added. Track labels can collapse; individual tracks can minimize or change height. Visibility, mute and lock controls live in the track settings sheet.
- Interactive volume fade graphs and duration presets for audio clips and the audio in video clips.
- Fixed the phone quick Delete action; retained undo for removed clips. Clip movement finds space instead of deleting overlapping clips, and selected clips have inset trim handles.
- Project library recovery, confirmed trash/restore and permanent deletion, `.procut` project import/export, and clip replacement. Media and imported LUTs can be removed; media can be sorted.
- Fullscreen playback with transport controls and Android landscape/portrait handling; improved speed-curve layout.
- ARM64-only Android build workflow for this iteration. The Android app is named Procut (`app.procut.editor.mobile`).

The current iteration is **0.2.14**. See [Atlas implementation and validation](docs/PROCUT-ATLAS.md) and [mobile installation notes](docs/PROCUT-MOBILE.md). Download the signed ARM64 APK supplied with the build; CI artifacts may use temporary signing keys.

## Source and credit

The source remains available under **AGPL-3.0-or-later**; see [LICENSE](LICENSE). Existing upstream copyright notices, third-party licenses, and contributor credits are retained. Procut's design adaptation and Android changes do not claim authorship of the original editor.

For the original project, its documentation, and upstream development, visit **[jub0t/Concat](https://github.com/jub0t/Concat)**. Older translated READMEs in `docs/` describe the upstream editor and have not been rewritten as Procut release notes.
