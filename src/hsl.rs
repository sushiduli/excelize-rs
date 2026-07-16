//! HSL color utilities.
//!
//! This module corresponds to `hsl.go` in the Go implementation.

/// HSL color representation.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct HSL {
    /// Hue.
    pub h: f64,
    /// Saturation.
    pub s: f64,
    /// Lightness.
    pub l: f64,
}

/// Convert an RGB triple to HSL.
pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let f_r = f64::from(r) / 255.0;
    let f_g = f64::from(g) / 255.0;
    let f_b = f64::from(b) / 255.0;
    let max_val = f64::max(f64::max(f_r, f_g), f_b);
    let min_val = f64::min(f64::min(f_r, f_g), f_b);
    let l = (max_val + min_val) / 2.0;
    let mut h;
    let s;
    if max_val == min_val {
        // Achromatic.
        h = 0.0;
        s = 0.0;
    } else {
        // Chromatic.
        let d = max_val - min_val;
        s = if l > 0.5 {
            d / (2.0 - max_val - min_val)
        } else {
            d / (max_val + min_val)
        };
        match max_val {
            x if x == f_r => {
                h = (f_g - f_b) / d;
                if f_g < f_b {
                    h += 6.0;
                }
            }
            x if x == f_g => {
                h = (f_b - f_r) / d + 2.0;
            }
            _ => {
                h = (f_r - f_g) / d + 4.0;
            }
        }
        h /= 6.0;
    }
    (h, s, l)
}

/// Convert an HSL triple to RGB.
pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let (f_r, f_g, f_b) = if s == 0.0 {
        (l, l, l)
    } else {
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - s * l
        };
        let p = 2.0 * l - q;
        (
            hue_to_rgb(p, q, h + 1.0 / 3.0),
            hue_to_rgb(p, q, h),
            hue_to_rgb(p, q, h - 1.0 / 3.0),
        )
    };
    let r = ((f_r * 255.0) + 0.5) as u8;
    let g = ((f_g * 255.0) + 0.5) as u8;
    let b = ((f_b * 255.0) + 0.5) as u8;
    (r, g, b)
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// Apply a tint to a base RGB color and return the result as `"FFRRGGBB"`.
///
/// `base_color` should be a 6-digit (`RRGGBB`) or 8-digit (`FFRRGGBB`) hex
/// string. For other inputs the original color is returned unchanged when the
/// tint is zero, or with an `FF` prefix otherwise.
pub fn theme_color(base_color: &str, tint: f64) -> String {
    if tint == 0.0 {
        return format!("FF{}", base_color);
    }

    let color = if base_color.len() == 8 {
        &base_color[2..]
    } else {
        base_color
    };

    if color.len() != 6 {
        return format!("FF{}", base_color);
    }

    let parse = |s: &str| u8::from_str_radix(s, 16).unwrap_or(0);
    let r = parse(&color[0..2]);
    let g = parse(&color[2..4]);
    let b = parse(&color[4..6]);

    let (h, s, mut l) = rgb_to_hsl(r, g, b);
    if tint < 0.0 {
        l *= 1.0 + tint;
    } else {
        l = l * (1.0 - tint) + (1.0 - (1.0 - tint));
    }

    let (br, bg, bb) = hsl_to_rgb(h, s, l);
    format!("FF{:02X}{:02X}{:02X}", br, bg, bb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_hsl_red() {
        let (h, s, l) = rgb_to_hsl(255, 0, 0);
        assert!((h - 0.0).abs() < 1e-10 || (h - 1.0).abs() < 1e-10);
        assert!((s - 1.0).abs() < 1e-10);
        assert!((l - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_rgb_to_hsl_green() {
        let (h, s, l) = rgb_to_hsl(0, 255, 0);
        assert!((h - 1.0 / 3.0).abs() < 1e-10);
        assert!((s - 1.0).abs() < 1e-10);
        assert!((l - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_rgb_to_hsl_blue() {
        let (h, s, l) = rgb_to_hsl(0, 0, 255);
        assert!((h - 2.0 / 3.0).abs() < 1e-10);
        assert!((s - 1.0).abs() < 1e-10);
        assert!((l - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_rgb_to_hsl_white() {
        let (h, s, l) = rgb_to_hsl(255, 255, 255);
        assert_eq!(h, 0.0);
        assert_eq!(s, 0.0);
        assert!((l - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rgb_to_hsl_black() {
        let (h, s, l) = rgb_to_hsl(0, 0, 0);
        assert_eq!(h, 0.0);
        assert_eq!(s, 0.0);
        assert_eq!(l, 0.0);
    }

    #[test]
    fn test_hsl_to_rgb_red() {
        assert_eq!(hsl_to_rgb(0.0, 1.0, 0.5), (255, 0, 0));
    }

    #[test]
    fn test_hsl_to_rgb_green() {
        assert_eq!(hsl_to_rgb(1.0 / 3.0, 1.0, 0.5), (0, 255, 0));
    }

    #[test]
    fn test_hsl_to_rgb_blue() {
        assert_eq!(hsl_to_rgb(2.0 / 3.0, 1.0, 0.5), (0, 0, 255));
    }

    #[test]
    fn test_hsl_to_rgb_white() {
        assert_eq!(hsl_to_rgb(0.0, 0.0, 1.0), (255, 255, 255));
    }

    #[test]
    fn test_hsl_to_rgb_black() {
        assert_eq!(hsl_to_rgb(0.0, 0.0, 0.0), (0, 0, 0));
    }

    #[test]
    fn test_hsl_round_trip() {
        for r in [0, 64, 128, 192, 255] {
            for g in [0, 64, 128, 192, 255] {
                for b in [0, 64, 128, 192, 255] {
                    let (h, s, l) = rgb_to_hsl(r, g, b);
                    let (rr, rg, rb) = hsl_to_rgb(h, s, l);
                    assert_eq!((r, g, b), (rr, rg, rb));
                }
            }
        }
    }

    #[test]
    fn test_theme_color() {
        assert_eq!(theme_color("000000", -0.1), "FF000000");
        assert_eq!(theme_color("000000", 0.0), "FF000000");
        assert_eq!(theme_color("00FF00", 0.2), "FF33FF33");
        assert_eq!(theme_color("000000", 1.0), "FFFFFFFF");
        assert_eq!(theme_color("FF0000", 1.0), "FFFFFFFF");
        assert_eq!(theme_color("FFFFFF", -1.0), "FF000000");
    }

    #[test]
    fn test_theme_color_eight_digit_input() {
        assert_eq!(theme_color("FF00FF00", 0.2), "FF33FF33");
    }

    #[test]
    fn test_theme_color_invalid_input() {
        assert_eq!(theme_color("red", 0.5), "FFred");
        assert_eq!(theme_color("red", 0.0), "FFred");
    }
}
