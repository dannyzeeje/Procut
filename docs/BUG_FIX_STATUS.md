# Bug-fix status

Source: jub0t/Concat at a7cda105c55cf4a5413dd747e16b0980ff2555c1, inspected on 2026-11-17.

This is an initial repair batch, not a resolution of the full backlog. The backlog below lists the 91 open issues labeled bug at inspection time. Unlabeled reports, feature requests, and future reports are outside this list. No upstream issue has been closed or commented on.

## Changes in this batch

- [#730](https://github.com/jub0t/Concat/issues/730): ignore an audio file's attached cover-art stream when probing media, so it is classified as audio and is not placed on the timeline as video. A generated 0.2-second MP3 fixture includes a tone and a red cover image; the regression test asserts audio is available and video is absent. Existing projects that already stored the file as a video clip need the media reimported; this change does not migrate saved clip kinds.
- [#596](https://github.com/jub0t/Concat/issues/596): include optional `bytes` in a completed export's job status. Record the size when the completion event arrives, retaining it if the file is later moved or deleted. Running jobs and files whose metadata cannot be read omit this field. A test checks serialization and retention after file removal.

## Existing source changes requiring runtime confirmation

- [#549](https://github.com/jub0t/Concat/issues/549): config-pane.slint already holds translated tab labels and updates them only when the tab strip changes, explicitly referencing this report. No new fix was made for this issue. The related section-strip report #768 remains unverified.

## Publication

The public fork is [dannyzeeje/Procut](https://github.com/dannyzeeje/Procut), forked from jub0t/Concat. This initial batch contains the two fixes listed above and the backlog status summary. Publication of this batch does not mean the remaining bug reports have been resolved.

## Validation

- Rust 1.93.1, the repository's pinned toolchain.
- `cargo test -p concat-core -p concat-project`: 349 unit tests and 2 documentation tests passed.
- `cargo test -p concat-media -p concat-api --lib`: 191 tests passed, 3 ignored, including both new regression tests.
- The media/API run used the checksum-verified FFmpeg 8.1 and ONNX Runtime 1.28.0 binaries in `scripts/ci/deps.lock`. Debian's system FFmpeg 7.1 produced 30 media-test failures involving unavailable encoders and filter options; all passed with the pinned build. ort-sys's default binary mirror returned HTTP 403; the pinned ONNX Runtime release supplied the build instead.
- Formatting and patch-whitespace checks passed.
- `cargo clippy -p concat-media -p concat-api --all-targets -- -D warnings` passed with the same pinned native dependencies.
- No desktop or mobile UI session, affected hardware, end-to-end packaged application, or saved-project migration was tested. The full application workspace was not built.

## Regression fixture

The cover-art fixture is generated media, with no third-party recording or image:

```sh
ffmpeg -f lavfi -i color=c=red:s=16x16 -frames:v 1 cover.jpg
ffmpeg -f lavfi -i sine=frequency=440:duration=0.2 -i cover.jpg \
  -map 0:a -map 1:v -c:a libmp3lame -c:v copy -disposition:v attached_pic \
  src/crates/concat-media/fixtures/audio-with-cover.mp3
```

## Remaining reports

These groups organize the backlog; they do not imply individual root-cause diagnosis. Device-specific reports need reproduction on the affected platform, relevant logs and sample media before a fix can be established.

## Crashes and device compatibility

- [#786](https://github.com/jub0t/Concat/issues/786) — Crash on export video. Open; not fixed or verified by this work.
- [#760](https://github.com/jub0t/Concat/issues/760) — 0.2.10 bug android. Open; not fixed or verified by this work.
- [#758](https://github.com/jub0t/Concat/issues/758) — Auto Caption Generation Crashes on Intel Mac (macOS Monterey) – Concat 0.2.10. Open; not fixed or verified by this work.
- [#747](https://github.com/jub0t/Concat/issues/747) — Export doesn't work. Open; not fixed or verified by this work.
- [#744](https://github.com/jub0t/Concat/issues/744) — Problema al exportar. Open; not fixed or verified by this work.
- [#743](https://github.com/jub0t/Concat/issues/743) — Android black preview . Open; not fixed or verified by this work.
- [#742](https://github.com/jub0t/Concat/issues/742) — video doesn't appear in preview. Open; not fixed or verified by this work.
- [#739](https://github.com/jub0t/Concat/issues/739) — High CPU usage and slow performance when using masks in a virtual machine. Open; not fixed or verified by this work.
- [#737](https://github.com/jub0t/Concat/issues/737) — Preview remains black / GPU device lost. Open; not fixed or verified by this work.
- [#733](https://github.com/jub0t/Concat/issues/733) — Windows 10 — Black preview and export fails with no GPU or software renderer (Intel UHD 620 + Radeon 530). Open; not fixed or verified by this work.
- [#712](https://github.com/jub0t/Concat/issues/712) — Relatório de erro: Ecrã de pré-visualização preto no Windows 10 com Intel HD Graphics 4600. Open; not fixed or verified by this work.
- [#709](https://github.com/jub0t/Concat/issues/709) — Title: Black screen and export error. Open; not fixed or verified by this work.
- [#708](https://github.com/jub0t/Concat/issues/708) — WARN  [concat-worker-0] fontdb: Failed to load a font face 0 from 'C:\WINDOWS\Fonts\mstmc.ttf' cause malformed font.. Open; not fixed or verified by this work.
- [#701](https://github.com/jub0t/Concat/issues/701) — The startup interface displays garbled text.. Open; not fixed or verified by this work.
- [#686](https://github.com/jub0t/Concat/issues/686) — Black screen. Open; not fixed or verified by this work.
- [#683](https://github.com/jub0t/Concat/issues/683) — The software has stability issues.. Open; not fixed or verified by this work.
- [#673](https://github.com/jub0t/Concat/issues/673) — Crash (STATUS_STACK_BUFFER_OVERRUN) when generating captions with whisper small model. Open; not fixed or verified by this work.
- [#654](https://github.com/jub0t/Concat/issues/654) — Please i need help. Open; not fixed or verified by this work.
- [#600](https://github.com/jub0t/Concat/issues/600) — macOS Intel (iMac14,1, Iris Pro): no window, Metal shader compiler error, CONCAT_GPU=software does not help. Open; not fixed or verified by this work.
- [#583](https://github.com/jub0t/Concat/issues/583) — Timeline Start Point dragging clips. Open; not fixed or verified by this work.
- [#575](https://github.com/jub0t/Concat/issues/575) — It crashes after adding a video and adding it to the timeline.. Open; not fixed or verified by this work.
- [#561](https://github.com/jub0t/Concat/issues/561) — UI Are Glitch / Not Rendering on Windows 11 on Concat-0.2.8 Version. Open; not fixed or verified by this work.
- [#556](https://github.com/jub0t/Concat/issues/556) — Concat 0.2.8 crashes on startup with NVIDIA RTX 5070 Ti (DXC shader compilation error). Open; not fixed or verified by this work.
- [#533](https://github.com/jub0t/Concat/issues/533) — Windows 11 Smart App Control blocks Concat at start: the bundled FFmpeg DLLs are unsigned. Open; not fixed or verified by this work.
- [#532](https://github.com/jub0t/Concat/issues/532) — Concat crashes on launch on macOS Ventura 13.7.8. Open; not fixed or verified by this work.
- [#523](https://github.com/jub0t/Concat/issues/523) — Crash program. Open; not fixed or verified by this work.
- [#509](https://github.com/jub0t/Concat/issues/509) — Concat Crash. Open; not fixed or verified by this work.
- [#507](https://github.com/jub0t/Concat/issues/507) — The app doesn't work on Xiaomi Redmi Note 14 5G; it shows a blank screen and then closes.. Open; not fixed or verified by this work.
- [#476](https://github.com/jub0t/Concat/issues/476) — feature: re-evaluate the GPU-based rendering and overall resource efficiency of the Slint user-interface and Concat app. Open; not fixed or verified by this work.
- [#449](https://github.com/jub0t/Concat/issues/449) — the screen it just been green. Open; not fixed or verified by this work.
- [#423](https://github.com/jub0t/Concat/issues/423) — macOS + legacy AMD GPU (GCN, OCLP): startup fails — wgpu requests -std=metal3.2, Metal compiler rejects it, no fallback adapter. Open; not fixed or verified by this work.
- [#395](https://github.com/jub0t/Concat/issues/395) — Concat Not Responding on Win 10. Open; not fixed or verified by this work.
- [#371](https://github.com/jub0t/Concat/issues/371) — App fails to render UI on Intel Iris Xe / Kubuntu 24.04. Open; not fixed or verified by this work.
- [#326](https://github.com/jub0t/Concat/issues/326) — Windows on ARM: concat.exe crashes at start (STATUS_ACCESS_VIOLATION on --version). Open; not fixed or verified by this work.
- [#294](https://github.com/jub0t/Concat/issues/294) — App crash at launch in IOS. Open; not fixed or verified by this work.
- [#242](https://github.com/jub0t/Concat/issues/242) — B R G (blue,red,green) Pixel App Glitch?. Open; not fixed or verified by this work.
- [#131](https://github.com/jub0t/Concat/issues/131) — Invalid surface. Open; not fixed or verified by this work.

## Exports

- [#766](https://github.com/jub0t/Concat/issues/766) — Critical bugs.... Open; not fixed or verified by this work.
- [#764](https://github.com/jub0t/Concat/issues/764) — EXPORT ERROR. Open; not fixed or verified by this work.
- [#754](https://github.com/jub0t/Concat/issues/754) — Incredible rendering bottleneck on a beast PC. Open; not fixed or verified by this work.
- [#750](https://github.com/jub0t/Concat/issues/750) — Export destination folder and resolution do not consistently sync with the initialized project settings. Open; not fixed or verified by this work.
- [#720](https://github.com/jub0t/Concat/issues/720) — Export fails when a source clip or image is renamed. Open; not fixed or verified by this work.
- [#704](https://github.com/jub0t/Concat/issues/704) — terrible video import. Open; not fixed or verified by this work.
- [#536](https://github.com/jub0t/Concat/issues/536) — Concat 0.28 export fails on video and photo. Open; not fixed or verified by this work.

## Captions and text

- [#763](https://github.com/jub0t/Concat/issues/763) — captions: 0.2.10 generates only 2 captions from a 1:30 clip (works on 0.2.8). Open; not fixed or verified by this work.
- [#680](https://github.com/jub0t/Concat/issues/680) — Can't edit a long text. Open; not fixed or verified by this work.
- [#656](https://github.com/jub0t/Concat/issues/656) — 使用过程中发现两个问题. Open; not fixed or verified by this work.
- [#652](https://github.com/jub0t/Concat/issues/652) — The background, the keyframe  and the Text got cutted off when expanding the title.. Open; not fixed or verified by this work.
- [#632](https://github.com/jub0t/Concat/issues/632) — SOME BUGS AND WISHS. Open; not fixed or verified by this work.
- [#623](https://github.com/jub0t/Concat/issues/623) — bug: text in preview bounding box overflows and clips. Open; not fixed or verified by this work.
- [#597](https://github.com/jub0t/Concat/issues/597) — Default title font "Cabinet Grotesk" silently renders as a fallback sans, and no tool reports the font used (0.2.8). Open; not fixed or verified by this work.

## Distribution and installation

- [#780](https://github.com/jub0t/Concat/issues/780) — Bug install on android. Open; not fixed or verified by this work.
- [#773](https://github.com/jub0t/Concat/issues/773) — Three download sources point at URLs that do not exist: the `models-v1` mirror was never published, `HF_REPO` is empty, and the sound manifest path has never existed. Open; not fixed or verified by this work.
- [#745](https://github.com/jub0t/Concat/issues/745) — Linux (pacman): in-app update fails with "xdg-open failed" because no app opens .pkg.tar.zst. Open; not fixed or verified by this work.
- [#705](https://github.com/jub0t/Concat/issues/705) — 苹果手机安装完软件后没有图标，图标变成灰色的. Open; not fixed or verified by this work.
- [#525](https://github.com/jub0t/Concat/issues/525) — نصب نشدن روی موبایل. Open; not fixed or verified by this work.
- [#499](https://github.com/jub0t/Concat/issues/499) — Ubuntu DEB installation not possible to create new project as normal user. Open; not fixed or verified by this work.
- [#479](https://github.com/jub0t/Concat/issues/479) — nix build can't finish even with 16GB memory. Open; not fixed or verified by this work.

## Editing, media and interface

- [#802](https://github.com/jub0t/Concat/issues/802) — Sudden Crashes after updating to 0.2.11 or 0.2.10. Open; not fixed or verified by this work.
- [#801](https://github.com/jub0t/Concat/issues/801) — captions editor cutting off first part of the first word. Open; not fixed or verified by this work.
- [#800](https://github.com/jub0t/Concat/issues/800) — Mouse wheel crash on Android. Open; not fixed or verified by this work.
- [#799](https://github.com/jub0t/Concat/issues/799) — Unable to copy and paste in the captions editor. Open; not fixed or verified by this work.
- [#790](https://github.com/jub0t/Concat/issues/790) — Flicker / single-frame black flash at cut points when concatenating multiple clips. Open; not fixed or verified by this work.
- [#781](https://github.com/jub0t/Concat/issues/781) — [Windows] concat.exe --mcp exits immediately without starting MCP stdio server. Open; not fixed or verified by this work.
- [#768](https://github.com/jub0t/Concat/issues/768) — Can't switch Basic / Enhance / Adjust tab when the video is playing in preview. Open; not fixed or verified by this work.
- [#762](https://github.com/jub0t/Concat/issues/762) — Audio goes silent when clip speed is changed (any speed, any file). Open; not fixed or verified by this work.
- [#696](https://github.com/jub0t/Concat/issues/696) — ⚠️An important bug. (CTRL+A) and (SHIFT+mouse) don't work in the Media sidebar to select all media.. Open; not fixed or verified by this work.
- [#692](https://github.com/jub0t/Concat/issues/692) — Open File dialog is not modal and can leave the app unresponsive. Open; not fixed or verified by this work.
- [#676](https://github.com/jub0t/Concat/issues/676) — When you play/open a project only to exit and open another project, it makes a black screen. Open; not fixed or verified by this work.
- [#641](https://github.com/jub0t/Concat/issues/641) — Video becomes out of sync / glitches after cutting clips. Open; not fixed or verified by this work.
- [#640](https://github.com/jub0t/Concat/issues/640) — 图片使用基本缩放，视频中的图片变得模糊，本身图片清晰度是比较高的，放大后应该清晰才对，现在. Open; not fixed or verified by this work.
- [#598](https://github.com/jub0t/Concat/issues/598) — MCP project_create into an unwritable folder reports "cannot find the file specified (os error 2)" instead of the real cause. Open; not fixed or verified by this work.
- [#592](https://github.com/jub0t/Concat/issues/592) — Bug Report: Preview Screen Not Fitting to Full Screen / Display Issue in concat. Open; not fixed or verified by this work.
- [#580](https://github.com/jub0t/Concat/issues/580) — Custom bg removal problem. Open; not fixed or verified by this work.
- [#572](https://github.com/jub0t/Concat/issues/572) — Installation issue on Android. Open; not fixed or verified by this work.
- [#546](https://github.com/jub0t/Concat/issues/546) — Same issue as Capcut, Use too much ram on low 8Gb Ram PC.. Open; not fixed or verified by this work.
- [#497](https://github.com/jub0t/Concat/issues/497) — Timeline has a bug at the end curve of the cut a black screen appears for 1 second. Open; not fixed or verified by this work.
- [#472](https://github.com/jub0t/Concat/issues/472) — bug: playback scrubbing not updating fast enough. Open; not fixed or verified by this work.
- [#462](https://github.com/jub0t/Concat/issues/462) — [Bug] Transition video playback stall (slideshow), clip split stutter, and Inspector UI glitches (v0.2.8). Open; not fixed or verified by this work.
- [#457](https://github.com/jub0t/Concat/issues/457) — bug: .mov neither have a preview nor have a functional preview on the timeline when played.. Open; not fixed or verified by this work.
- [#448](https://github.com/jub0t/Concat/issues/448) — Transitions effects are all the same, just a fade. Open; not fixed or verified by this work.
- [#432](https://github.com/jub0t/Concat/issues/432) — (Low priority) Cut points are off by one frame vs. the timestamp with iPhone HEVC footage. Open; not fixed or verified by this work.
- [#430](https://github.com/jub0t/Concat/issues/430) — Filling a template slot with a shorter video keeps the old clip length (black tail). Open; not fixed or verified by this work.
- [#351](https://github.com/jub0t/Concat/issues/351) — Editor: empty band between the preview and the timeline. Open; not fixed or verified by this work.
- [#343](https://github.com/jub0t/Concat/issues/343) — preview window does not play the video in Ubuntu 24.0. Open; not fixed or verified by this work.
- [#340](https://github.com/jub0t/Concat/issues/340) — Display bug. Open; not fixed or verified by this work.
- [#285](https://github.com/jub0t/Concat/issues/285) — image gets stretched to canvas and doesn't stay inside transformation control. Open; not fixed or verified by this work.
- [#149](https://github.com/jub0t/Concat/issues/149) — Linux/rpm - Bug: Lags, Unstable, etc.. Open; not fixed or verified by this work.
