// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer Abdullah

//! Reverse: the span a clip covers of a media file, written backwards,
//! once.
//!
//! Playing a file backwards live means seeking to every frame in turn,
//! and each seek is a decode from the keyframe before it: choppy at best,
//! and wrong where a seek lands short of its frame. So a reverse is not
//! played, it is written. The span the clip covers is read forwards once,
//! turned round, and saved as a project media file of its own,
//! picture and sound, which the clip then shows like any other file. The
//! import is never touched; the bin keeps it.
//!
//! Turning a span round without holding it whole - a minute of 4K is
//! tens of gigabytes of pixels - takes two passes. The first reads the
//! span forwards in runs of a few hundred megabytes, writes each run
//! backwards into a segment of its own, and moves on. The second reads
//! the segments last to first into the copy. Every frame is decoded and
//! encoded twice for spans that exceed a run, the segments near-losslessly,
//! and nothing is sought but the span's start. The sound goes through the mixer's own graph with
//! `areverse`, which holds the span's samples - a few hundred megabytes
//! an hour - and is muxed in beside the picture.
//!
//! One job at a time through a [`SingleFlight`], like every long job the
//! host runs. The copy is named after the file's path, size and
//! modification time and the span it covers, so a file replaced on disk
//! gets a fresh one and the same span asked for twice gets the one
//! already there.

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use concat_core::{Frame, FrameRate, Rational};
use concat_media::audio::{AudioClip, mix_to_file, mux};
use concat_media::{
    DecodeOptions, Decoder, EncodeOptions, Encoder, FrameSink, FrameSource, RateMode, VideoCodec,
};

use crate::jobs::SingleFlight;

mod flow;

/// What one reverse covers.
#[derive(Clone, Debug)]
pub struct ReverseRequest {
    /// The media file to read.
    pub media_path: String,
    /// A sound clip: no picture to turn, only the samples.
    pub audio_only: bool,
    /// The source stream the clip plays, or its default stream.
    pub audio_stream: Option<usize>,
    /// Seconds into the file where the span begins.
    pub start: f64,
    /// Seconds of the file the span covers.
    pub duration: f64,
    /// Where the copy goes; see [`target_for`].
    pub target: PathBuf,
}

/// The reverse service: the one-job slot.
pub struct Reversers {
    gate: Arc<SingleFlight>,
}

/// Pixels a run of frames may hold before it is written out as a segment:
/// a hundred-odd frames of 1080p, a couple of dozen of 4K.
const RUN_BYTES: usize = if cfg!(target_os = "android") {
    96 << 20
} else {
    384 << 20
};

/// Where the reversed copy of `media`'s span lives under `project`: in
/// its processed media, named by the file's path, size and modification time and
/// the span in milliseconds. None for a file that cannot be stat'ed,
/// which cannot be read either.
pub fn target_for(
    project: &Path,
    media: &str,
    start: f64,
    duration: f64,
    audio_only: bool,
) -> Option<PathBuf> {
    let meta = std::fs::metadata(media).ok()?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    media.hash(&mut hasher);
    meta.len().hash(&mut hasher);
    if let Ok(modified) = meta.modified()
        && let Ok(since) = modified.duration_since(std::time::UNIX_EPOCH)
    {
        since.as_secs().hash(&mut hasher);
    }
    // Internal Android copies carry PCM so neither operation depends on
    // MediaCodec accepting and draining a separate AAC encode.
    let extension = match (cfg!(target_os = "android"), audio_only) {
        (true, true) => "wav",
        (true, false) => "mov",
        (false, true) => "m4a",
        (false, false) => "mp4",
    };
    Some(project.join("media").join("processed").join(format!(
        "reverse-v3-{:016x}-{}-{}.{extension}",
        hasher.finish(),
        (start * 1000.0).round() as u64,
        (duration * 1000.0).round() as u64
    )))
}

impl Default for Reversers {
    fn default() -> Self {
        Self::new()
    }
}

impl Reversers {
    /// A service with its slot free.
    pub fn new() -> Reversers {
        Reversers {
            gate: Arc::new(SingleFlight::new()),
        }
    }

    /// Whether reverse or frame interpolation is running.
    pub fn is_busy(&self) -> bool {
        self.gate.is_busy()
    }

    /// Asks the running job to stop after the frame in hand.
    pub fn cancel(&self) {
        self.gate.cancel();
    }

    /// Render a higher-frame-rate copy of the clip's source span.
    pub fn interpolate(
        &self,
        request: &ReverseRequest,
        factor: u32,
        progress: &mut dyn FnMut(f32),
    ) -> Result<PathBuf, String> {
        if request.audio_only
            || !request.start.is_finite()
            || !request.duration.is_finite()
            || request.duration <= 0.0
        {
            return Err("nothing to interpolate".to_owned());
        }
        let job = self.gate.begin("optical flow")?;
        if let Some(folder) = request.target.parent() {
            std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
        }
        if request.target.is_file() && check_picture(&request.target).is_ok() {
            return Ok(request.target.clone());
        }
        progress(0.0);
        flow::write(request, factor, &job, progress)?;
        progress(1.0);
        Ok(request.target.clone())
    }

    /// Writes the copy `request` asks for, or finds it already written.
    /// Blocks until done, so run it on its own thread; `progress` is
    /// called with a fraction as it goes. Returns where the copy is.
    pub fn reverse(
        &self,
        request: &ReverseRequest,
        progress: &mut dyn FnMut(f32),
    ) -> Result<PathBuf, String> {
        let job = self.gate.begin("reverse")?;
        if request.target.is_file() {
            let readable = if request.audio_only {
                concat_media::probe(&request.target).is_ok_and(|info| info.audio.is_some())
            } else {
                check_picture(&request.target).is_ok()
            };
            if readable {
                return Ok(request.target.clone());
            }
            std::fs::remove_file(&request.target).map_err(|error| error.to_string())?;
        }
        if !request.start.is_finite() || !request.duration.is_finite() || request.duration <= 0.0 {
            return Err("nothing to reverse".to_owned());
        }
        if let Some(folder) = request.target.parent() {
            std::fs::create_dir_all(folder)
                .map_err(|error| format!("{}: {error}", folder.display()))?;
        }
        progress(0.0);
        let result = if request.audio_only {
            sound(request, &job)
        } else {
            footage(request, &job, progress)
        };
        result.map(|()| {
            progress(1.0);
            request.target.clone()
        })
    }
}

/// The span as the mixer sees it, with `chain` on it.
fn span_clip(request: &ReverseRequest, chain: &str) -> AudioClip {
    AudioClip {
        path: PathBuf::from(&request.media_path),
        stream: request.audio_stream,
        start: 0.0,
        duration: request.duration,
        source_start: request.start,
        speed: 1.0,
        tempo: Vec::new(),
        preserve_pitch: true,
        volume: 1.0,
        volume_curve: concat_core::animate::Track::new(Vec::new()),
        fade_in: 0.0,
        fade_out: 0.0,
        smooth_fade_in: false,
        smooth_fade_out: false,
        filter_chain: chain.to_owned(),
    }
}

/// Check the generated stream before letting a clip point at it.
fn check_picture(path: &Path) -> Result<(), String> {
    let options = DecodeOptions::default().in_software().scaled_to(32, 32);
    let mut decoder = Decoder::open(path, &options).map_err(|error| error.to_string())?;
    if decoder
        .next_frame()
        .map_err(|error| error.to_string())?
        .is_none()
    {
        return Err("the generated video has no readable frames".to_owned());
    }
    Ok(())
}

/// A partial file moved into place, or cleared away with the error.
fn land(partial: &Path, target: &Path, written: Result<(), String>) -> Result<(), String> {
    let result = written.and_then(|()| {
        std::fs::rename(partial, target).map_err(|error| format!("{}: {error}", target.display()))
    });
    if result.is_err() {
        let _ = std::fs::remove_file(partial);
    }
    result
}

fn staged(target: &Path, stage: &str) -> PathBuf {
    target.with_extension(format!(
        "{stage}.{}",
        target
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("mp4")
    ))
}

fn check_cancel(job: &crate::jobs::Job) -> Result<(), String> {
    if job.cancelled() {
        return Err("clip processing cancelled".to_owned());
    }
    Ok(())
}

/// Attach only the clip's chosen stream, and leave the previous copy in
/// place until a new one can be opened and decoded.
fn finish_copy(
    request: &ReverseRequest,
    picture: &Path,
    has_audio: bool,
    chain: &str,
    job: &crate::jobs::Job,
) -> Result<(), String> {
    if !has_audio {
        return land(
            picture,
            &request.target,
            check_cancel(job).and_then(|()| check_picture(picture)),
        );
    }
    let audio_extension = if request.target.extension().is_some_and(|ext| ext == "mov") {
        "part.wav"
    } else {
        "part.m4a"
    };
    let audio = request.target.with_extension(audio_extension);
    let muxed = staged(&request.target, "muxed.part");
    let result = check_cancel(job)
        .and_then(|()| {
            mix_to_file(&[span_clip(request, chain)], request.duration, &audio)
                .map_err(|error| error.to_string())
        })
        .and_then(|()| check_cancel(job))
        .and_then(|()| mux(picture, &audio, &muxed).map_err(|error| error.to_string()))
        .and_then(|()| check_cancel(job))
        .and_then(|()| check_picture(&muxed));
    let _ = std::fs::remove_file(picture);
    let _ = std::fs::remove_file(&audio);
    land(&muxed, &request.target, result)
}

/// The span's sound, backwards, as a file of its own: the mixer's graph
/// with `areverse` on it.
fn sound(request: &ReverseRequest, job: &crate::jobs::Job) -> Result<(), String> {
    let partial = staged(&request.target, "part");
    let written = check_cancel(job)
        .and_then(|()| {
            mix_to_file(
                &[span_clip(request, "areverse")],
                request.duration,
                &partial,
            )
            .map_err(|error| error.to_string())
        })
        .and_then(|()| check_cancel(job));
    land(&partial, &request.target, written)
}

/// The span's picture, backwards, with its sound beside it when the file
/// has any. Written under partial names and moved into place only when
/// whole, so a copy that is there is a copy that is complete.
fn footage(
    request: &ReverseRequest,
    job: &crate::jobs::Job,
    progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
    let info = concat_media::probe(&request.media_path).map_err(|error| error.to_string())?;
    let video = info
        .video
        .as_ref()
        .ok_or_else(|| "no video stream".to_owned())?;
    // MPEG-4 Part 2 needs a timebase denominator at most 65,535.
    let rate = if video.frame_rate.fps().numerator() > 65_535 {
        FrameRate::from_int(video.frame_rate.fps().as_f64().round().max(1.0) as u32)
    } else {
        video.frame_rate
    };
    // About how many frames the span holds, for the readout and for a
    // container that keeps no clock.
    let expected = frame_count(request.duration, rate);
    // Even sides, which the encoder wants: a stray pixel at the right or
    // bottom is cropped rather than the whole copy refused.
    let width = video.width & !1;
    let height = video.height & !1;
    if width == 0 || height == 0 {
        return Err("the picture is too small to reverse".to_owned());
    }
    let partial_video = staged(&request.target, "part");

    // Pass one: the span forwards, in runs, each run written backwards as
    // a segment of its own.
    let start = Rational::approximate(request.start).unwrap_or(Rational::ZERO);
    let options = DecodeOptions::default()
        .in_software()
        .threaded(2)
        .starting_at(start)
        .at_rate(rate)
        .repeating();
    let mut decoder =
        Decoder::open(&request.media_path, &options).map_err(|error| error.to_string())?;
    let frame_bytes = width as usize * height as usize * concat_core::frame::BYTES_PER_PIXEL;
    let run = (RUN_BYTES / frame_bytes.max(1)).clamp(2, 240);
    let mut segments: Vec<PathBuf> = Vec::new();
    let mut held: Vec<Frame> = Vec::with_capacity(run);
    let mut count = 0u64;
    let passes = (|| -> Result<(), String> {
        while count < expected {
            check_cancel(job)?;
            let Some(frame) = decoder.next_frame().map_err(|error| error.to_string())? else {
                break;
            };
            if frame.width() < width || frame.height() < height {
                return Err("the video resolution changed during reverse".to_owned());
            }
            held.push(crate::enhance::crop(&frame, width, height));
            count += 1;
            if held.len() >= run && count < expected {
                write_segment(request, &mut segments, &mut held, rate)?;
            }
            progress(0.5 * count as f32 / expected as f32);
        }
        drop(decoder);
        if count == 0 {
            return Err("the span holds no frames".to_owned());
        }

        // Pass two: the segments last to first, into the copy.
        let mut encoder = Encoder::create(&partial_video, width, height, rate, &copy_options())
            .map_err(|error| error.to_string())?;
        let mut written = 0u64;
        let encoded = (|| -> Result<(), String> {
            // The last run is already in memory, so encode it straight into
            // the final copy. For a short clip this is the entire span.
            for frame in held.drain(..).rev() {
                check_cancel(job)?;
                encoder
                    .write_frame(&frame)
                    .map_err(|error| error.to_string())?;
                written += 1;
                progress(0.5 + 0.45 * written as f32 / count as f32);
            }
            for segment in segments.iter().rev() {
                let mut reader =
                    Decoder::open(segment, &DecodeOptions::default().in_software().threaded(2))
                        .map_err(|error| error.to_string())?;
                while let Some(frame) = reader.next_frame().map_err(|error| error.to_string())? {
                    check_cancel(job)?;
                    encoder
                        .write_frame(&frame)
                        .map_err(|error| error.to_string())?;
                    written += 1;
                    progress(0.5 + 0.45 * written as f32 / count as f32);
                }
            }
            if written != count {
                return Err(format!("reverse wrote {written} of {count} frames"));
            }
            encoder.finish().map_err(|error| error.to_string())
        })();
        if let Err(error) = encoded {
            let _ = std::fs::remove_file(&partial_video);
            return Err(error);
        }
        Ok(())
    })();
    for segment in &segments {
        let _ = std::fs::remove_file(segment);
    }
    if let Err(error) = passes {
        let _ = std::fs::remove_file(&partial_video);
        return Err(error);
    }
    finish_copy(
        request,
        &partial_video,
        info.audio.is_some(),
        "areverse",
        job,
    )
}

fn frame_count(duration: f64, rate: FrameRate) -> u64 {
    // A span cut at an exact frame can be a few floating point bits past it.
    (duration.mul_add(rate.fps().as_f64(), -1e-7))
        .ceil()
        .max(1.0) as u64
}

/// The run in `held`, backwards, as the next segment; the run is emptied
/// either way.
fn write_segment(
    request: &ReverseRequest,
    segments: &mut Vec<PathBuf>,
    held: &mut Vec<Frame>,
    rate: FrameRate,
) -> Result<(), String> {
    let path = request
        .target
        .with_extension(format!("seg{:04}.part.mp4", segments.len()));
    let Some(first) = held.first() else {
        return Ok(());
    };
    let (width, height) = (first.width(), first.height());
    let written = Encoder::create(&path, width, height, rate, &segment_options())
        .and_then(|mut encoder| {
            for frame in held.iter().rev() {
                encoder.write_frame(frame)?;
            }
            encoder.finish()
        })
        .map_err(|error| error.to_string());
    held.clear();
    match written {
        Ok(()) => {
            segments.push(path);
            Ok(())
        }
        Err(error) => {
            let _ = std::fs::remove_file(&path);
            Err(error)
        }
    }
}

/// A segment lives minutes and is read once: the working codec at a
/// quality nothing is lost through.
fn segment_options() -> EncodeOptions {
    EncodeOptions {
        codec: VideoCodec::Mpeg4,
        preset: "ultrafast".to_owned(),
        crf: 8,
        rate_mode: RateMode::Vbr,
        bitrate_kbps: 0,
        ten_bit: false,
        color_range: concat_media::ColorRange::Limited,
        hardware: false,
        threads: 2,
        software_only: false,
    }
}

/// The copy the timeline reads for as long as the project lives:
/// near-transparent quality, as the enhanced copy is written, on the
/// machine's own encoder as an export is - several times x264's speed at
/// the same picture.
fn copy_options() -> EncodeOptions {
    EncodeOptions {
        codec: if cfg!(target_os = "android") {
            VideoCodec::Mpeg4
        } else {
            VideoCodec::H264
        },
        preset: "medium".to_owned(),
        crf: 16,
        rate_mode: RateMode::Vbr,
        bitrate_kbps: 0,
        ten_bit: false,
        color_range: concat_media::ColorRange::Limited,
        hardware: !cfg!(target_os = "android"),
        threads: 2,
        software_only: cfg!(target_os = "android"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_copy_is_named_after_the_file_and_the_span() {
        let dir = std::env::temp_dir().join(format!("concat-reverse-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let media = dir.join("clip.mp4");
        std::fs::write(&media, b"not really").expect("write");
        let media = media.to_string_lossy().into_owned();
        let project = dir.join("project");
        let span = target_for(&project, &media, 1.5, 4.0, false).expect("named");
        assert!(span.starts_with(project.join("media").join("processed")));
        assert!(
            span.file_stem()
                .unwrap()
                .to_string_lossy()
                .ends_with("-1500-4000")
        );
        assert_eq!(
            span.extension().unwrap(),
            if cfg!(target_os = "android") {
                "mov"
            } else {
                "mp4"
            }
        );
        assert_eq!(
            target_for(&project, &media, 1.5, 4.0, false),
            Some(span.clone())
        );
        assert_ne!(
            target_for(&project, &media, 2.0, 4.0, false),
            Some(span.clone())
        );
        let sound = target_for(&project, &media, 1.5, 4.0, true).expect("named");
        assert_eq!(
            sound.extension().unwrap(),
            if cfg!(target_os = "android") {
                "wav"
            } else {
                "m4a"
            }
        );
        assert_eq!(
            target_for(
                &project,
                &dir.join("gone.mp4").to_string_lossy(),
                0.0,
                1.0,
                false
            ),
            None
        );
        // A file with different bytes is a different copy.
        std::fs::write(&media, b"not really, but longer").expect("write");
        assert_ne!(target_for(&project, &media, 1.5, 4.0, false), Some(span));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_run_is_sized_to_the_frame() {
        let hd = RUN_BYTES / (1920 * 1080 * 4);
        let uhd = RUN_BYTES / (3840 * 2160 * 4);
        assert!(hd > uhd && uhd >= 2, "{hd} {uhd}");
        assert!((RUN_BYTES / (7680 * 4320 * 4)).clamp(2, 240) <= 3);
    }
}
