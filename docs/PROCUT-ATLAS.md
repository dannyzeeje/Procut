# Procut 0.2.14 — Design Atlas iteration

This fork adapts the supplied Procut Design Atlas to Android. Its purple palette and diamond mark replace the Atlas's earlier green/P branding. The source logo is `assets/procut-logo.svg`; Android uses matching vector paths, and legacy icon sizes are rendered from that SVG. Startup presents the symbol, Procut name, and an animated progress line.

## Phone interaction

Projects has a compact header, Import and Find project actions, search/sort, project rows, and a fixed New project button. The menu contains project export, library recovery, trash, and settings. Import/export uses `.procut` packages containing a saved project and its assets. Trash operations retain their confirmation and restore behavior.

The timeline toolbar controls track-label collapse, snapping, timeline settings, and adding media. Tap a track label to show visibility, mute, lock, minimization, and track height controls. Minimization changes the view only: the clip still plays and exports. The drag divider reallocates room between preview and timeline. Tools open as grouped pill buttons; inspector sheets have Close and a pinned Done action. Fullscreen playback retains the existing native landscape behavior.

New phone projects start with two tracks, V1 above A1. The document stores audio/video roles; automatic placement uses a free lane of the matching kind and creates one when necessary. Titles/effects/filters use additional lanes. Existing projects retain their tracks, and manually placing media on a specific track remains supported. Video and audio numbers are separate.

Volume fades are available for audio clips and videos with sound. Drag the graph's two control points to change fade durations, or choose None, Quick, Soft, Long, Fade in, or Fade out. These presets set durations, using the engine's existing quadratic gain fades; they do not introduce arbitrary automation curves. The volume slider remains available. Both fields commit as one undoable edit and apply to preview/export.

The quick Delete button now invokes the existing clip deletion callback directly. The compatibility shortcut also routes Delete. Normal undo restores deleted clips.

## Build and installation

This iteration builds only `arm64-v8a`; the former x86 emulator companion is disabled. Therefore this version does not claim a new emulator smoke pass. The previous 0.2.13 Android 11/15 startup, landscape fullscreen, portrait return, and export smoke results remain historical evidence only.

The delivered APK is signed with the same private release key as the delivered 0.2.13 Procut APK, under `app.procut.editor.mobile`. It can update that installation without uninstalling it. The earlier Procut Next app uses another identifier; keep it installed until any projects have been transferred. Private signing material is stored outside the repository and is never included in an APK download or source commit.

Validation covers model and project tests, mobile routing/deletion/undo regressions, locale inventory, Slint compilation, and offscreen checks of production phone components. Build and signature verification are performed before delivery. Offscreen checks are not a physical-device test.

## Attribution

Original editor: [Concat](https://github.com/jub0t/Concat), by [Jareer Abdullah / jub0t](https://github.com/jub0t) and contributors. Procut changes are described in the root README; upstream copyright and AGPL licensing are preserved.

Suggested GitHub About description: “Android-focused fork of jub0t/Concat: Procut Atlas UI, purple branding, multitrack controls, project packages, and mobile fixes. Original editor by Jareer Abdullah.”
