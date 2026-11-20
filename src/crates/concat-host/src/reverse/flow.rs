// SPDX-License-Identifier: AGPL-3.0-or-later

//! Offline optical flow for a constant-speed slow-motion clip.
use super::*;
use concat_vision::stabilize::track::{Gray, Pyramid};

const COLS: usize = 12;
const ROWS: usize = 8;
type Vector = (f32, f32);

/// Track a grid at a bounded analysis resolution. Reject vectors that do
/// not track back to their starting point, such as points across a cut.
fn vectors(a: &Frame, b: &Frame) -> Option<[Vector; COLS * ROWS]> {
    let (width, height) = (a.width() as f32, a.height() as f32);
    let scale = (384.0 / width.max(height)).min(1.0);
    let gray = |frame: &Frame| {
        let w = ((width * scale).round() as usize).max(1);
        let h = ((height * scale).round() as usize).max(1);
        let mut gray = Gray::filled(w, h, 0.0);
        for y in 0..h {
            for x in 0..w {
                let px = ((x as f32 / scale) as u32).min(frame.width() - 1);
                let py = ((y as f32 / scale) as u32).min(frame.height() - 1);
                let [r, g, b, _] = frame.pixel(px, py).expect("inside frame");
                gray.data[y * w + x] =
                    0.299 * f32::from(r) + 0.587 * f32::from(g) + 0.114 * f32::from(b);
            }
        }
        Pyramid::of(gray)
    };
    let left = gray(a);
    let right = gray(b);
    let mut grid = [None; COLS * ROWS];
    let mut valid = 0;
    for (i, vector) in grid.iter_mut().enumerate() {
        let x = (i % COLS) as f32 * (left.base().width - 1) as f32 / (COLS - 1) as f32;
        let y = (i / COLS) as f32 * (left.base().height - 1) as f32 / (ROWS - 1) as f32;
        if let Some(end) = left.flow(&right, (x, y))
            && let Some(back) = right.flow(&left, end)
            && (back.0 - x).hypot(back.1 - y) < 1.5
        {
            *vector = Some(((end.0 - x) / scale, (end.1 - y) / scale));
            valid += 1;
        }
    }
    if valid < 6 {
        return None;
    }
    Some(std::array::from_fn(|i| {
        // Borrow the nearest reliable vector for low-texture cells.
        grid[i].unwrap_or_else(|| {
            grid.iter()
                .enumerate()
                .filter_map(|(j, v)| {
                    v.map(|v| {
                        (
                            (i / COLS).abs_diff(j / COLS) + (i % COLS).abs_diff(j % COLS),
                            v,
                        )
                    })
                })
                .min_by_key(|&(distance, _)| distance)
                .map_or((0.0, 0.0), |(_, vector)| vector)
        })
    }))
}

fn between(a: &Frame, b: &Frame, flow: &[Vector; COLS * ROWS], t: f32) -> Frame {
    let (w, h) = (a.width(), a.height());
    let mut picture = a.clone();
    let pixels = picture.pixels_mut();
    for y in 0..h {
        let gy = y as f32 * (ROWS - 1) as f32 / (h - 1).max(1) as f32;
        let y0 = gy.floor() as usize;
        let y1 = (y0 + 1).min(ROWS - 1);
        let fy = gy.fract();
        for x in 0..w {
            let gx = x as f32 * (COLS - 1) as f32 / (w - 1).max(1) as f32;
            let x0 = gx.floor() as usize;
            let x1 = (x0 + 1).min(COLS - 1);
            let fx = gx.fract();
            let lerp = |u: f32, v: f32, f: f32| u + (v - u) * f;
            let [p, q, r, s] = [
                flow[y0 * COLS + x0],
                flow[y0 * COLS + x1],
                flow[y1 * COLS + x0],
                flow[y1 * COLS + x1],
            ];
            let dx = lerp(lerp(p.0, q.0, fx), lerp(r.0, s.0, fx), fy);
            let dy = lerp(lerp(p.1, q.1, fx), lerp(r.1, s.1, fx), fy);
            let sample = |frame: &Frame, weight: f32| {
                let sx = (x as f32 + dx * weight).clamp(0.0, (w - 1) as f32);
                let sy = (y as f32 + dy * weight).clamp(0.0, (h - 1) as f32);
                let x0 = sx as u32;
                let y0 = sy as u32;
                let x1 = (x0 + 1).min(w - 1);
                let y1 = (y0 + 1).min(h - 1);
                let [p, q, r, s] = [
                    frame.pixel(x0, y0).unwrap(),
                    frame.pixel(x1, y0).unwrap(),
                    frame.pixel(x0, y1).unwrap(),
                    frame.pixel(x1, y1).unwrap(),
                ];
                std::array::from_fn::<_, 4, _>(|c| {
                    lerp(
                        lerp(f32::from(p[c]), f32::from(q[c]), sx.fract()),
                        lerp(f32::from(r[c]), f32::from(s[c]), sx.fract()),
                        sy.fract(),
                    )
                })
            };
            let left = sample(a, -t);
            let right = sample(b, 1.0 - t);
            let offset = ((y * w + x) * 4) as usize;
            for c in 0..4 {
                pixels[offset + c] = lerp(left[c], right[c], t).round() as u8;
            }
        }
    }
    picture
}

/// Keep the generated copy within a portable encoder clock and cap
/// its sustained decoding and storage cost on phones.
fn interpolation_rates(source: FrameRate, factor: u32) -> (FrameRate, u32, FrameRate) {
    let input = source.fps().as_f64().round().clamp(1.0, 60.0) as u32;
    let factor = factor.clamp(2, (120 / input).min(8));
    (
        FrameRate::from_int(input),
        factor,
        FrameRate::from_int(input * factor),
    )
}

pub(super) fn write(
    request: &ReverseRequest,
    factor: u32,
    stream: Option<usize>,
    job: &crate::jobs::Job,
) -> Result<(), String> {
    let info = concat_media::probe(&request.media_path).map_err(|e| e.to_string())?;
    let source_rate = info.video.as_ref().ok_or("no video stream")?.frame_rate;
    let (rate, factor, output) = interpolation_rates(source_rate, factor);
    let options = DecodeOptions::default()
        .in_software()
        .threaded(2)
        .starting_at(Rational::approximate(request.start).unwrap_or(Rational::ZERO))
        .at_rate(rate);
    let mut decoder = Decoder::open(&request.media_path, &options).map_err(|e| e.to_string())?;
    let first = decoder
        .next_frame()
        .map_err(|e| e.to_string())?
        .ok_or("no video frames")?;
    let (width, height) = (first.width() & !1, first.height() & !1);
    if width == 0 || height == 0 {
        return Err("video is too small to interpolate".to_owned());
    }
    let mut left = crate::enhance::crop(&first, width, height);
    let partial = request.target.with_extension("part.mp4");
    let pcm = request.target.with_extension("part.m4a");
    let muxed = request.target.with_extension("muxed.part.mp4");
    let result = (|| {
        let mut encoder = Encoder::create(&partial, width, height, output, &copy_options())
            .map_err(|e| e.to_string())?;
        let frames = (request.duration * output.fps().as_f64()).ceil().max(1.0) as u64;
        let mut written = 0;
        while written < frames {
            if job.cancelled() {
                return Err("optical flow cancelled".to_owned());
            }
            let right = decoder.next_frame().map_err(|e| e.to_string())?;
            if right
                .as_ref()
                .is_some_and(|frame| frame.width() < width || frame.height() < height)
            {
                return Err("the video resolution changed during interpolation".to_owned());
            }
            let right = right.map(|frame| crate::enhance::crop(&frame, width, height));
            let flow = right.as_ref().and_then(|r| vectors(&left, r));
            for i in 0..factor {
                if written == frames {
                    break;
                }
                if job.cancelled() {
                    return Err("optical flow cancelled".to_owned());
                }
                let fraction = i as f32 / factor as f32;
                let frame = match (&right, &flow) {
                    (Some(r), Some(v)) if i > 0 => between(&left, r, v, fraction),
                    (Some(r), _) if fraction >= 0.5 => r.clone(),
                    _ => left.clone(),
                };
                encoder.write_frame(&frame).map_err(|e| e.to_string())?;
                written += 1;
            }
            if let Some(right) = right {
                left = right;
            }
        }
        encoder.finish().map_err(|e| e.to_string())?;
        if info.audio.is_none() {
            return land(&partial, &request.target, check_picture(&partial));
        }
        let mut audio = span_clip(request, "");
        audio.stream = stream;
        mix_to_file(&[audio], request.duration, &pcm)
            .and_then(|()| mux(&partial, &pcm, &muxed))
            .map_err(|e| e.to_string())?;
        land(&muxed, &request.target, check_picture(&muxed))
    })();
    for path in [&partial, &pcm, &muxed] {
        let _ = std::fs::remove_file(path);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractional_high_rates_stay_within_the_software_encoders_clock() {
        let source = FrameRate::new(Rational::new(60_000, 1001));
        let (input, factor, output) = interpolation_rates(source, 8);
        assert_eq!(input, FrameRate::from_int(60));
        assert_eq!(factor, 2);
        assert_eq!(output, FrameRate::from_int(120));
    }

    #[test]
    fn an_intermediate_picture_warps_motion_instead_of_blending_ghosts() {
        let mut a = Frame::black(32, 16);
        let mut b = Frame::black(32, 16);
        for y in 0..16 {
            a.set_pixel(8, y, [255, 255, 255, 255]);
            b.set_pixel(12, y, [255, 255, 255, 255]);
        }
        let mid = between(&a, &b, &[(4.0, 0.0); COLS * ROWS], 0.5);
        assert_eq!(mid.pixel(10, 8), Some([255, 255, 255, 255]));
        assert_eq!(mid.pixel(8, 8).unwrap()[0], 0);
        assert_eq!(mid.pixel(12, 8).unwrap()[0], 0);
    }
}
