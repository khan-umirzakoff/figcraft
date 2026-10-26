//! Pixel-integrated triangular blur, with floating passes and one terminal byte resolve.

use tiny_skia::Pixmap;

/// Finite support `radius` in device pixels; transparent samples outside the surface.
pub(crate) fn blur(p: &mut Pixmap, radius: f64) {
    if !radius.is_finite() || radius <= 0.5 {
        return;
    }
    let radius = radius.min(crate::fx::MAX_SIGMA * 2.0);
    let (w, h) = (p.width() as usize, p.height() as usize);
    for channel in 0..4 {
        let source: Vec<f32> = p.data().as_chunks::<4>().0.iter().map(|px| f32::from(px[channel]) / 255.0).collect();
        let horizontal = axis(&source, w, h, radius, false);
        drop(source);
        let output = axis(&horizontal, w, h, radius, true);
        for (px, value) in p.data_mut().as_chunks_mut::<4>().0.iter_mut().zip(output) {
            px[channel] = (value * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
}

fn axis(source: &[f32], width: usize, height: usize, radius: f64, vertical: bool) -> Vec<f32> {
    let edge = (radius + 0.5).floor() as usize;
    let interior = edge.saturating_sub(1);
    let square = radius * radius;
    let edge_weight = 0.5 * (1.0 - (edge as f64 - 0.5) / radius).powi(2);
    let (lines, length) = if vertical { (width, height) } else { (height, width) };
    let mut output = vec![0.0; source.len()];
    let mut prefix = vec![(0.0, 0.0); length + 1];
    for line in 0..lines {
        let index = |x| if vertical { x * width + line } else { line * width + x };
        let sample = |x| f64::from(source.get(index(x)).copied().unwrap_or(0.0));
        let (mut sum, mut weighted) = (0.0, 0.0);
        for (x, entry) in prefix.iter_mut().skip(1).enumerate() {
            let value = sample(x);
            sum += value;
            weighted += x as f64 * value;
            *entry = (sum, weighted);
        }
        for position in 0..length {
            let left = position.saturating_sub(interior);
            let right = position.saturating_add(interior).min(length.saturating_sub(1)) + 1;
            let get = |i| prefix.get(i).copied().unwrap_or_default();
            let (ls, lw) = get(left);
            let (ms, mw) = get(position + 1);
            let (rs, rw) = get(right);
            let x = position as f64;
            // The centre tap straddles the cusp; the two end taps cover fractional pixels.
            let mut value = ((radius - x) * (ms - ls) + mw - lw + (radius + x) * (rs - ms) - rw + mw - sample(position) * 0.25) / square;
            if position >= edge {
                value += sample(position - edge) * edge_weight;
            }
            if position.saturating_add(edge) < length {
                value += sample(position + edge) * edge_weight;
            }
            if let Some(out) = output.get_mut(index(position)) {
                *out = value.clamp(0.0, 1.0) as f32;
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cdf(x: f64) -> f64 {
        if x <= -1.0 {
            0.0
        } else if x < 0.0 {
            0.5 * (x + 1.0).powi(2)
        } else if x < 1.0 {
            1.0 - 0.5 * (1.0 - x).powi(2)
        } else {
            1.0
        }
    }

    #[test]
    fn weighted_prefix_matches_direct_integrated_kernel() {
        let source: Vec<f32> = (0..35).map(|i| ((i * 13) % 31) as f32 / 31.0).collect();
        for radius in [0.5001, 0.8, 1.0, 1.3, 2.0, 3.7, 12.0, 1000.0, 2_000_000.0] {
            for vertical in [false, true] {
                let (lines, length): (usize, usize) = if vertical { (7, 5) } else { (5, 7) };
                let actual = axis(&source, 7, 5, radius, vertical);
                for line in 0..lines {
                    let index = |x| if vertical { x * 7 + line } else { line * 7 + x };
                    for x in 0..length {
                        let expected: f64 = (0..length)
                            .map(|i| {
                                let d = x as f64 - i as f64;
                                f64::from(source[index(i)]) * (cdf((d + 0.5) / radius) - cdf((d - 0.5) / radius))
                            })
                            .sum();
                        assert!((f64::from(actual[index(x)]) - expected).abs() < 1e-6, "{radius}, {vertical}, {line}, {x}");
                    }
                }
            }
        }
    }

    #[test]
    fn two_passes_resolve_once_and_keep_premultiplied_colour() {
        let mut p = Pixmap::new(7, 5).unwrap();
        for (i, px) in p.data_mut().as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let a = ((i * 13) % 256) as u8;
            *px = [a, a / 2, 0, a];
        }
        let original = p.clone();
        for radius in [0.0, 0.5, f64::NAN, f64::INFINITY, -1.0] {
            blur(&mut p, radius);
            assert_eq!(p.data(), original.data());
        }
        let radius = 1.3;
        blur(&mut p, radius);
        for y in 0..5 {
            for x in 0..7 {
                for channel in 0..4 {
                    let expected: f64 = (0..5)
                        .flat_map(|j| (0..7).map(move |i| (i, j)))
                        .map(|(i, j)| {
                            let dx = x as f64 - i as f64;
                            let dy = y as f64 - j as f64;
                            f64::from(original.data()[(j * 7 + i) * 4 + channel])
                                * (cdf((dx + 0.5) / radius) - cdf((dx - 0.5) / radius))
                                * (cdf((dy + 0.5) / radius) - cdf((dy - 0.5) / radius))
                        })
                        .sum();
                    assert_eq!(p.data()[(y * 7 + x) * 4 + channel], expected.round() as u8);
                }
            }
        }
        assert!(p.data().as_chunks::<4>().0.iter().all(|px| px[..3].iter().all(|v| *v <= px[3])));
    }
}
