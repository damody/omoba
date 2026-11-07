//! Selective HSV recolour of an 8-bit PNG (build-time art variants, e.g. a hero's kimono tint).
//! The output keeps the source dimensions, channel layout and alpha; a pixel changes only when it
//! matches a rule's hue/saturation/value window (first matching rule wins), so UV atlases stay intact.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::BufWriter;

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    /// Inclusive hue window in degrees; min > max wraps through 0 (e.g. 330..15).
    pub hue: (f32, f32),
    pub saturation: (f32, f32),
    pub value: (f32, f32),
    pub target_hue: f32,
    /// Floor of the new saturation; the source saturation adds half its own amount on top.
    pub target_saturation: f32,
    pub value_scale: f32,
}

fn hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue = if delta <= f32::EPSILON {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta).rem_euclid(6.0))
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let saturation = if max <= f32::EPSILON { 0.0 } else { delta / max };
    (hue, saturation, max)
}

fn rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h.rem_euclid(360.0) / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let byte = |value: f32| ((value + m).clamp(0.0, 1.0) * 255.0).round() as u8;
    (byte(r), byte(g), byte(b))
}

fn hue_in(hue: f32, (min, max): (f32, f32)) -> bool {
    if min <= max { hue >= min && hue <= max } else { hue >= min || hue <= max }
}

/// Recolours RGB(A) pixels in place; `channels` is 3 or 4. Returns the number of changed pixels.
pub fn recolor_pixels(pixels: &mut [u8], channels: usize, rules: &[Rule]) -> usize {
    let mut changed = 0;
    for pixel in pixels.chunks_exact_mut(channels) {
        if channels == 4 && pixel[3] == 0 {
            continue;
        }
        let (h, s, v) = hsv(pixel[0], pixel[1], pixel[2]);
        let Some(rule) = rules.iter().find(|rule| {
            hue_in(h, rule.hue) && s >= rule.saturation.0 && s <= rule.saturation.1 && v >= rule.value.0 && v <= rule.value.1
        }) else {
            continue;
        };
        let saturation = (rule.target_saturation + 0.5 * s).min(1.0);
        let (r, g, b) = rgb(rule.target_hue, saturation, (v * rule.value_scale).min(1.0));
        if (r, g, b) != (pixel[0], pixel[1], pixel[2]) {
            pixel[0] = r;
            pixel[1] = g;
            pixel[2] = b;
            changed += 1;
        }
    }
    changed
}

fn number(value: &Value, key: &str, min: f64, max: f64) -> Result<f32, String> {
    let n = value.get(key).and_then(Value::as_f64).ok_or_else(|| format!("rule.{key} must be a number"))?;
    if !n.is_finite() || n < min || n > max {
        return Err(format!("rule.{key} must be within {min}..{max}"));
    }
    Ok(n as f32)
}

fn window(value: &Value, key: &str, max: f64) -> Result<(f32, f32), String> {
    let pair = value.get(key).and_then(Value::as_array).filter(|pair| pair.len() == 2)
        .ok_or_else(|| format!("rule.{key} must be [min, max]"))?;
    let read = |index: usize| -> Result<f32, String> {
        let n = pair[index].as_f64().filter(|n| n.is_finite() && *n >= 0.0 && *n <= max)
            .ok_or_else(|| format!("rule.{key} bounds must be within 0..{max}"))?;
        Ok(n as f32)
    };
    Ok((read(0)?, read(1)?))
}

pub fn parse_rules(value: &Value) -> Result<Vec<Rule>, String> {
    let rules = value.as_array().filter(|rules| !rules.is_empty() && rules.len() <= 16)
        .ok_or("rules must be a non-empty array of at most 16 rules")?;
    rules.iter().map(|rule| {
        let saturation = window(rule, "saturation", 1.0)?;
        let value = window(rule, "value", 1.0)?;
        if saturation.0 > saturation.1 || value.0 > value.1 {
            return Err("saturation/value windows must be ascending".to_string());
        }
        Ok(Rule {
            hue: window(rule, "hue", 360.0)?,
            saturation,
            value,
            target_hue: number(rule, "target_hue", 0.0, 360.0)?,
            target_saturation: number(rule, "target_saturation", 0.0, 1.0)?,
            value_scale: number(rule, "value_scale", 0.1, 2.0)?,
        })
    }).collect()
}

pub fn run(params: &Value) -> Result<Value, String> {
    let source = params.get("source").and_then(Value::as_str).ok_or("source required")?;
    let output = params.get("output").and_then(Value::as_str).ok_or("output required")?;
    if source == output {
        return Err("output must differ from source (the source texture stays untouched)".into());
    }
    let rules = parse_rules(params.get("rules").unwrap_or(&Value::Null))?;
    let mut decoder = png::Decoder::new(File::open(source).map_err(|e| format!("{source}: {e}"))?);
    decoder.set_transformations(png::Transformations::IDENTITY);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let info = reader.info().clone();
    if info.bit_depth != png::BitDepth::Eight {
        return Err("only 8-bit PNG is supported".into());
    }
    let channels = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        other => return Err(format!("only RGB/RGBA PNG is supported, got {other:?}")),
    };
    if info.width == 0 || info.height == 0 || info.width > 16384 || info.height > 16384 {
        return Err("PNG dimensions out of bounds".into());
    }
    let mut pixels = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut pixels).map_err(|e| e.to_string())?;
    pixels.truncate(frame.buffer_size());
    let changed = recolor_pixels(&mut pixels, channels, &rules);
    {
        let file = File::create(output).map_err(|e| format!("{output}: {e}"))?;
        let mut encoder = png::Encoder::new(BufWriter::new(file), info.width, info.height);
        encoder.set_color(info.color_type);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(&pixels).map_err(|e| e.to_string())?;
    }
    let bytes = std::fs::read(output).map_err(|e| e.to_string())?;
    Ok(json!({
        "width": info.width, "height": info.height, "channels": channels,
        "pixels": (info.width as u64) * (info.height as u64), "changed": changed,
        "output_sha256": format!("{:x}", Sha256::digest(&bytes)),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pink_rule() -> Rule {
        Rule { hue: (330.0, 15.0), saturation: (0.0, 0.4), value: (0.55, 1.0), target_hue: 345.0, target_saturation: 0.55, value_scale: 0.9 }
    }

    #[test]
    fn only_matching_pixels_change_and_layout_and_alpha_stay() {
        // white, pale pink, skin (hue ~25), gold (hue ~45), black hair, transparent white
        let mut pixels = vec![
            250, 250, 250, 255, 240, 205, 210, 255, 225, 180, 150, 255,
            210, 170, 90, 255, 30, 20, 20, 255, 250, 250, 250, 0,
        ];
        let original = pixels.clone();
        let changed = recolor_pixels(&mut pixels, 4, &[pink_rule()]);
        assert_eq!(changed, 2);
        assert_eq!(pixels.len(), original.len());
        for (index, (after, before)) in pixels.chunks(4).zip(original.chunks(4)).enumerate() {
            assert_eq!(after[3], before[3], "alpha of pixel {index} changed");
            if index >= 2 { assert_eq!(after, before, "pixel {index} outside the window changed"); }
        }
        let (h, s, _) = hsv(pixels[0], pixels[1], pixels[2]);
        assert!((h - 345.0).abs() < 2.0 && s >= 0.5, "white became {h} {s}");
    }

    #[test]
    fn hue_window_wraps_and_rejects_bad_rules() {
        assert!(hue_in(350.0, (330.0, 15.0)) && hue_in(5.0, (330.0, 15.0)) && !hue_in(25.0, (330.0, 15.0)));
        assert!(parse_rules(&json!([])).is_err());
        assert!(parse_rules(&json!([{"hue":[0,10],"saturation":[0.5,0.1],"value":[0,1],"target_hue":1,"target_saturation":0.5,"value_scale":1}])).is_err());
        assert!(parse_rules(&json!([{"hue":[330,15],"saturation":[0,0.4],"value":[0.55,1],"target_hue":345,"target_saturation":0.55,"value_scale":0.9}])).is_ok());
    }

    #[test]
    fn hsv_round_trips() {
        for (r, g, b) in [(250u8, 250u8, 250u8), (200, 30, 80), (10, 200, 40), (0, 0, 0)] {
            let (h, s, v) = hsv(r, g, b);
            let (r2, g2, b2) = rgb(h, s, v);
            assert!((r as i32 - r2 as i32).abs() <= 1 && (g as i32 - g2 as i32).abs() <= 1 && (b as i32 - b2 as i32).abs() <= 1);
        }
    }
}
