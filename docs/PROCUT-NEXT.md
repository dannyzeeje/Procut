# Procut Next 0.2.12

This mobile iteration adds:

- A full-width speed-ramp editor with a readable minimum graph height and spaced axis labels.
- Left track controls for video visibility, mute, lock, and Auto/Small/Medium/Large row heights.
- A draggable divider between transport and timeline. The timeline starts larger and can use up to 70% of the editor.
- Inset selection trim grips, snapping and magnetic-trim controls.
- Single-clip touch insert moves: crowded drops choose a nearby cut edge and push following clips right. Clips are neither overwritten nor deleted; Undo reverses the whole move. The source gap remains. Multiple selected clips retain the existing safe-gap placement.
- Remove buttons and Added/Name/Type sorting in the mobile media bin. Timeline-used assets remain protected; remove their clips first. Original source files are never deleted from the bin.
- Remove imported LUTs from the Adjustments sheet. LUTs used in the current project's timelines must be detached first; factory looks remain available.
- Library rediscovery for project folders omitted from recents, plus confirmed move-to-trash, tap-to-restore, and confirmed permanent deletion from Trash.
- Portable `.procut` packages, including imported media, fonts, and custom LUT tables. Export from Project; import from the home screen. Import creates a new folder and unique LUT ids. Archive traversal, symlinks, duplicate entries and oversized expansion are rejected. Third-party shader/effect packages are not embedded.
- Replace Clip from the selected clip's tool strip, preserving its position, length and effects. Replacement footage must have the same type and enough duration for the clip's speed settings.
- A minimal turquoise Pc app mark.

## Install and existing projects

The Android package is `app.procut.editor.next`, labelled **Procut Next**, so this preview installs beside the earlier Procut preview. The earlier build used a temporary signing key that is unavailable to this build. Keep the earlier installation: Android isolates its private projects from the new app. Library rediscovers projects in this installation's current and default project locations; it cannot recover another app's private files. On desktop, existing project folders can be opened using the folder button. On mobile, use Library for this installation’s project folders and Import for portable `.procut` packages; raw folder browsing is not supported. Project packages can be moved between installations that support export/import.

Preview builds use temporary CI signing keys. A permanent private release signing key must be configured before promising in-place updates across future builds.

## Design references

- [Blackmagic Design: DaVinci Resolve 19.1 new features guide](https://documents.blackmagicdesign.com/SupportNotes/DaVinci_Resolve_19_1_New_Features_Guide.pdf?_v=1731398410000): track headers and lock/mute/visibility control conventions.
- [Blackmagic Design: DaVinci Resolve 20.2 new features guide](https://documents.blackmagicdesign.com/SupportNotes/DaVinci_Resolve_20.2_New_Features_Guide.pdf?_v=1757487610000): track and ripple editing controls.
- [LumaFusion reference guide](https://www.luma-touch.com/wp-content/uploads/2020/07/LumaFusion-Reference-Guide.pdf): mobile preview/timeline sizing, track switches, insert edits and project archives.

These features adapt familiar editing behavior to the existing Procut engine; they do not reproduce those editors' full feature sets.

## Verification

Local checks: full Slint UI generation, locale inventory, 258 project-model tests, and 39 host project tests. The latter cover trash/restore/recovery, portable media/custom-LUT round trips, and archive traversal rejection. Android CI compiles ARM64 and an x86_64 emulator companion, then performs startup and video-export smoke checks on Android 11 and 15.
