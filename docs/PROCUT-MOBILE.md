# Procut 0.2.13 — Android interface

- A compact mobile app bar, a welcoming home screen, and spaced search/sort/project rows.
- The hamburger menu contains My projects, Import project, Export project, Find saved projects, Trash and Settings. Export project asks which project to package, opens it and writes a portable `.procut` file. Import opens the phone's file picker.
- In the editor, Export video and Export project are separate menu actions. The four/five essential bottom actions stay visible. More tools opens a labeled vertical list, including every former sideways toolbar action.
- The timeline divider has its own space above a 32 dp ruler, so it cannot cover timestamps or the playhead marker. Touch timestamps use shorter labels and the playhead has a larger marker and a thicker line.
- Each layer has a label and a bounded two-by-two control grid. Small/Medium/Large heights are 88/112/144 dp; the row-size popup no longer stretches through the layer.
- Full-screen preview is a dedicated playback-only view, with play/pause, frame stepping, seeking, time readouts and Back. Android hides its system bars, rotates to sensor landscape, then returns to portrait editing on exit or Back.
- A new rounded-square P/play icon, including Android adaptive icon resources.

## Installation and signing

This package is `app.procut.editor.mobile`, labelled **Procut**. It installs beside the earlier Procut Next preview, whose temporary CI signing key is unavailable. Keep that installation and its projects. Private app data does not migrate automatically; transfer projects using a `.procut` package where available.

The APK delivered from the workspace is re-signed with a persistent private Procut key held outside the public repository. Keep a secure backup of that key and its password for future in-place updates. GitHub Actions artifacts use temporary test signatures unless the repository's release-signing secrets are configured; those test artifacts cannot update the persistently signed installation.

## Verification

The phone UI is generated independently and rendered at narrow and regular phone sizes, plus a landscape full-screen view. Project package/recovery tests and core lints run locally. Android CI builds ARM64 and an x86_64 emulator companion. Its Android 11/15 export smoke run also enters landscape full-screen preview and returns to portrait before exporting video, and checks responsiveness throughout.
