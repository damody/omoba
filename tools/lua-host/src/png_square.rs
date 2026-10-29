//! Square icon from an 8-bit or 16-bit PNG of any colour type (build-time import of original
//! icons and portraits, whose sizes and aspect ratios vary): the centred square crop, resampled
//! to `edge` pixels (area average when shrinking, bilinear when enlarging), written as RGBA.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::BufWriter;

/// Decoded image as RGBA8.
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Baseline or progressive JPEG (some originals are JPEG whatever their extension says).
fn decode_jpeg(source: &str, bytes: &[u8]) -> Result<Rgba, String> {
    let mut decoder = jpeg_decoder::Decoder::new(bytes);
    let data = decoder.decode().map_err(|e| format!("{source}: {e}"))?;
    let info = decoder.info().ok_or_else(|| format!("{source}: JPEG without frame info"))?;
    let (width, height) = (info.width as u32, info.height as u32);
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return Err("JPEG dimensions out of bounds".into());
    }
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    match info.pixel_format {
        jpeg_decoder::PixelFormat::L8 => for &l in &data { pixels.extend_from_slice(&[l, l, l, 255]); },
        jpeg_decoder::PixelFormat::L16 => for l in data.chunks_exact(2) { pixels.extend_from_slice(&[l[0], l[0], l[0], 255]); },
        jpeg_decoder::PixelFormat::RGB24 => for p in data.chunks_exact(3) { pixels.extend_from_slice(&[p[0], p[1], p[2], 255]); },
        jpeg_decoder::PixelFormat::CMYK32 => for p in data.chunks_exact(4) {
            // Adobe CMYK JPEGs store inverted values: channel = 255 - ink
            let k = p[3] as u32;
            let channel = |c: u8| ((c as u32 * k) / 255) as u8;
            pixels.extend_from_slice(&[channel(p[0]), channel(p[1]), channel(p[2]), 255]);
        },
    }
    if pixels.len() != width as usize * height as usize * 4 {
        return Err(format!("{source}: JPEG pixel data does not match its size"));
    }
    Ok(Rgba { width, height, pixels })
}

pub fn decode(source: &str) -> Result<Rgba, String> {
    let bytes = std::fs::read(source).map_err(|e| format!("{source}: {e}"))?;
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return decode_jpeg(source, &bytes);
    }
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| format!("{source}: {e}"))?;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut buffer).map_err(|e| format!("{source}: {e}"))?;
    let (width, height) = (frame.width, frame.height);
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return Err("PNG dimensions out of bounds".into());
    }
    let channels = match frame.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        other => return Err(format!("unsupported colour type after expansion: {other:?}")),
    };
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    for row in 0..height as usize {
        let line = &buffer[row * frame.line_size..row * frame.line_size + width as usize * channels];
        for px in line.chunks_exact(channels) {
            let rgba = match channels {
                1 => [px[0], px[0], px[0], 255],
                2 => [px[0], px[0], px[0], px[1]],
                3 => [px[0], px[1], px[2], 255],
                _ => [px[0], px[1], px[2], px[3]],
            };
            pixels.extend_from_slice(&rgba);
        }
    }
    Ok(Rgba { width, height, pixels })
}

/// Centred square crop of `image`, resampled to `edge` x `edge`.
pub fn square(image: &Rgba, edge: u32) -> Vec<u8> {
    let side = image.width.min(image.height) as f64;
    let x0 = (image.width as f64 - side) / 2.0;
    let y0 = (image.height as f64 - side) / 2.0;
    let scale = side / edge as f64;
    let at = |x: i64, y: i64, c: usize| -> f64 {
        let x = x.clamp(0, image.width as i64 - 1) as usize;
        let y = y.clamp(0, image.height as i64 - 1) as usize;
        image.pixels[(y * image.width as usize + x) * 4 + c] as f64
    };
    let mut out = Vec::with_capacity(edge as usize * edge as usize * 4);
    for oy in 0..edge {
        for ox in 0..edge {
            let mut rgba = [0.0f64; 4];
            if scale > 1.0 {
                // area average over the source footprint of this pixel
                let (sx0, sy0) = (x0 + ox as f64 * scale, y0 + oy as f64 * scale);
                let (sx1, sy1) = (sx0 + scale, sy0 + scale);
                let mut weight = 0.0;
                let mut y = sy0.floor();
                while y < sy1 {
                    let wy = (sy1.min(y + 1.0) - sy0.max(y)).max(0.0);
                    let mut x = sx0.floor();
                    while x < sx1 {
                        let wx = (sx1.min(x + 1.0) - sx0.max(x)).max(0.0);
                        let w = wx * wy;
                        let alpha = at(x as i64, y as i64, 3) / 255.0;
                        for c in 0..3 { rgba[c] += at(x as i64, y as i64, c) * w * alpha; }
                        rgba[3] += alpha * w;
                        weight += w;
                        x += 1.0;
                    }
                    y += 1.0;
                }
                // premultiplied: colour divided by covered alpha, alpha by area
                let covered = rgba[3];
                for c in 0..3 { rgba[c] = if covered > 0.0 { rgba[c] / covered } else { 0.0 }; }
                rgba[3] = if weight > 0.0 { covered / weight * 255.0 } else { 0.0 };
            } else {
                let sx = x0 + (ox as f64 + 0.5) * scale - 0.5;
                let sy = y0 + (oy as f64 + 0.5) * scale - 0.5;
                let (fx, fy) = (sx.floor(), sy.floor());
                let (tx, ty) = (sx - fx, sy - fy);
                for c in 0..4 {
                    let top = at(fx as i64, fy as i64, c) * (1.0 - tx) + at(fx as i64 + 1, fy as i64, c) * tx;
                    let bottom = at(fx as i64, fy as i64 + 1, c) * (1.0 - tx) + at(fx as i64 + 1, fy as i64 + 1, c) * tx;
                    rgba[c] = top * (1.0 - ty) + bottom * ty;
                }
            }
            for c in 0..4 { out.push(rgba[c].round().clamp(0.0, 255.0) as u8); }
        }
    }
    out
}

pub fn run(params: &Value) -> Result<Value, String> {
    let source = params.get("source").and_then(Value::as_str).ok_or("source required")?;
    let output = params.get("output").and_then(Value::as_str).ok_or("output required")?;
    if source == output {
        return Err("output must differ from source".into());
    }
    let edge = params.get("edge").and_then(Value::as_u64).ok_or("edge required")?;
    if !(16..=2048).contains(&edge) {
        return Err("edge must be 16..2048".into());
    }
    let image = decode(source)?;
    let pixels = square(&image, edge as u32);
    {
        let file = File::create(output).map_err(|e| format!("{output}: {e}"))?;
        let mut encoder = png::Encoder::new(BufWriter::new(file), edge as u32, edge as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(&pixels).map_err(|e| e.to_string())?;
    }
    let bytes = std::fs::read(output).map_err(|e| e.to_string())?;
    let source_bytes = std::fs::read(source).map_err(|e| e.to_string())?;
    Ok(json!({
        "source_width": image.width, "source_height": image.height, "edge": edge,
        "source_sha256": format!("{:x}", Sha256::digest(&source_bytes)),
        "output_sha256": format!("{:x}", Sha256::digest(&bytes)), "output_bytes": bytes.len(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(width: u32, height: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Rgba {
        let mut pixels = Vec::new();
        for y in 0..height { for x in 0..width { pixels.extend_from_slice(&f(x, y)); } }
        Rgba { width, height, pixels }
    }

    #[test]
    fn crops_the_centre_of_a_wide_image() {
        // 4x2: red, green, green, blue columns; the centred square is the two green columns
        let wide = image(4, 2, |x, _| match x { 0 => [255, 0, 0, 255], 3 => [0, 0, 255, 255], _ => [0, 255, 0, 255] });
        let out = square(&wide, 2);
        assert!(out.chunks_exact(4).all(|p| p == [0, 255, 0, 255]));
    }

    #[test]
    fn shrinking_averages_and_enlarging_keeps_flat_colour() {
        let checker = image(4, 4, |x, y| if (x + y) % 2 == 0 { [255, 255, 255, 255] } else { [0, 0, 0, 255] });
        let out = square(&checker, 2);
        assert!(out.chunks_exact(4).all(|p| (p[0] as i32 - 128).abs() <= 1 && p[3] == 255));
        let flat = image(2, 2, |_, _| [10, 20, 30, 255]);
        assert!(square(&flat, 8).chunks_exact(4).all(|p| p == [10, 20, 30, 255]));
    }

    #[test]
    fn decodes_jpeg_by_content_not_extension() {
        // a tracked repository JPEG, found from this crate upwards
        let rel = "docs/acceptance/2026-10-19-new-machine/video-400.jpg";
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).ancestors()
            .map(|dir| dir.join(rel)).find(|p| p.is_file()).expect("repository JPEG fixture");
        let dir = std::env::temp_dir().join(format!("png_square_jpeg_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let disguised = dir.join("photo.png");
        std::fs::copy(&source, &disguised).unwrap();
        let image = decode(disguised.to_str().unwrap()).unwrap();
        assert!(image.width > 0 && image.height > 0);
        assert!(image.pixels.chunks_exact(4).all(|p| p[3] == 255));
        let out = dir.join("out.png");
        let report = run(&json!({"source": disguised.to_str(), "output": out.to_str(), "edge": 64})).unwrap();
        assert_eq!(report["edge"], 64);
        assert_eq!(decode(out.to_str().unwrap()).unwrap().width, 64);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn round_trips_through_files_and_rejects_bad_edges() {
        let dir = std::env::temp_dir().join(format!("png_square_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src.png");
        {
            let file = File::create(&src).unwrap();
            let mut encoder = png::Encoder::new(BufWriter::new(file), 3, 2);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header().unwrap().write_image_data(&[9; 18]).unwrap();
        }
        let out = dir.join("out.png");
        let report = run(&json!({"source": src.to_str(), "output": out.to_str(), "edge": 32})).unwrap();
        assert_eq!(report["source_width"], 3);
        let back = decode(out.to_str().unwrap()).unwrap();
        assert_eq!((back.width, back.height), (32, 32));
        assert!(back.pixels.chunks_exact(4).all(|p| p == [9, 9, 9, 255]));
        assert!(run(&json!({"source": src.to_str(), "output": out.to_str(), "edge": 4})).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }
}
