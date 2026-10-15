use agent_computer_use_core::{Error, Result};
use image::RgbaImage;

const SPACING: f64 = 100.0;

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy)]
struct Axis {
    origin: f64,
    extent: f64,
    pixels: usize,
}

impl Axis {
    fn new(origin: f64, extent: f64, pixels: usize) -> Result<Self> {
        let scale = pixels as f64 / extent;
        if !origin.is_finite()
            || !extent.is_finite()
            || extent <= 0.0
            || pixels == 0
            || !(origin + extent).is_finite()
            || !scale.is_finite()
            || scale * SPACING < 1.0
            || origin + SPACING == origin
        {
            return Err(platform_error("invalid screenshot coordinate geometry"));
        }
        Ok(Self {
            origin,
            extent,
            pixels,
        })
    }

    fn scale(self) -> f64 {
        self.pixels as f64 / self.extent
    }

    fn position(self, coordinate: f64) -> f64 {
        (coordinate - self.origin) * self.scale()
    }

    fn ticks(self) -> impl Iterator<Item = (f64, f64)> {
        let first = (self.origin / SPACING).ceil() * SPACING;
        let count = ((self.origin + self.extent - first) / SPACING)
            .ceil()
            .max(0.0) as usize;
        (0..count).map(move |index| {
            let coordinate = first + index as f64 * SPACING;
            (coordinate, self.position(coordinate))
        })
    }
}

fn platform_error(message: impl Into<String>) -> Error {
    Error::PlatformError {
        message: message.into(),
    }
}

/// Overlay absolute screen-coordinate grid without changing image dimensions.
///
/// `bounds` is the captured region in absolute screen coordinates (e.g.
/// window rect or primary screen origin). Grid lines are every 100 points;
/// labels show absolute coordinates usable directly with `click --x X --y Y`.
pub fn overlay(img: &mut RgbaImage, bounds: Bounds) -> Result<()> {
    let (width, height) = (img.width() as usize, img.height() as usize);
    let x = Axis::new(bounds.x, bounds.width, width)?;
    let y = Axis::new(bounds.y, bounds.height, height)?;

    for (_, position) in x.ticks() {
        let px = position.floor() as i32;
        vline(img, px);
    }
    for (_, position) in y.ticks() {
        let py = position.floor() as i32;
        hline(img, py);
    }

    let unit = (2.0 * x.scale().min(y.scale())).max(1.0).round() as u32;
    let unit = unit.max(1);
    let badge_h = 9 * unit;
    let mut right = label(img, x.origin, 2, 2, unit);
    for (coordinate, position) in x.ticks() {
        let start = position.floor() as u32 + 3;
        if start >= right + 4 {
            right = label(img, coordinate, start, 2, unit);
        }
    }
    let mut bottom = 2 * badge_h + 4;
    label(img, y.origin, 2, badge_h + 4, unit);
    for (coordinate, position) in y.ticks() {
        let start = position.floor() as u32 + 3;
        if start >= bottom + 4 {
            label(img, coordinate, 2, start, unit);
            bottom = start + badge_h;
        }
    }
    Ok(())
}

fn blend(dst: [u8; 4], src: [u8; 4]) -> [u8; 4] {
    let a = src[3] as f32 / 255.0;
    let inv = 1.0 - a;
    [
        (src[0] as f32 * a + dst[0] as f32 * inv).round() as u8,
        (src[1] as f32 * a + dst[1] as f32 * inv).round() as u8,
        (src[2] as f32 * a + dst[2] as f32 * inv).round() as u8,
        255,
    ]
}

fn put(img: &mut RgbaImage, x: i32, y: i32, color: [u8; 4]) {
    if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
        return;
    }
    let px = img.get_pixel_mut(x as u32, y as u32);
    let blended = blend(px.0, color);
    px.0 = blended;
}

fn fill_rect(img: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
    for dy in 0..h {
        for dx in 0..w {
            put(img, (x + dx) as i32, (y + dy) as i32, color);
        }
    }
}

fn vline(img: &mut RgbaImage, x: i32) {
    let h = img.height() as i32;
    for y in 0..h {
        // black shadow then white core, matching macOS readability
        put(img, x - 1, y, [0, 0, 0, 64]);
        put(img, x, y, [255, 255, 255, 140]);
    }
}

fn hline(img: &mut RgbaImage, y: i32) {
    let w = img.width() as i32;
    for x in 0..w {
        put(img, x, y - 1, [0, 0, 0, 64]);
        put(img, x, y, [255, 255, 255, 140]);
    }
}

fn number(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded == 0.0 {
        return "0".into();
    }
    format!("{rounded:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn label(img: &mut RgbaImage, coordinate: f64, x: u32, y: u32, unit: u32) -> u32 {
    let text = number(coordinate);
    let width = text.len() as u32 * 6 * unit + unit;
    let height = 9 * unit;
    fill_rect(img, x, y, width, height, [0, 0, 0, 217]);
    for (index, digit) in text.bytes().enumerate() {
        for (row, bits) in glyph(digit).into_iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    let px = x + (index as u32 * 6 + column + 1) * unit;
                    let py = y + (row as u32 + 1) * unit;
                    fill_rect(img, px, py, unit, unit, [255, 255, 255, 255]);
                }
            }
        }
    }
    x + width
}

fn glyph(digit: u8) -> [u8; 7] {
    match digit {
        b'0' => [14, 17, 19, 21, 25, 17, 14],
        b'1' => [4, 12, 4, 4, 4, 4, 14],
        b'2' => [14, 17, 1, 2, 4, 8, 31],
        b'3' => [30, 1, 1, 14, 1, 1, 30],
        b'4' => [2, 6, 10, 18, 31, 2, 2],
        b'5' => [31, 16, 16, 30, 1, 1, 30],
        b'6' => [14, 16, 16, 30, 17, 17, 14],
        b'7' => [31, 1, 2, 4, 8, 8, 8],
        b'8' => [14, 17, 17, 14, 17, 17, 14],
        b'9' => [14, 17, 17, 15, 1, 1, 14],
        b'-' => [0, 0, 0, 31, 0, 0, 0],
        b'.' => [0, 0, 0, 0, 0, 12, 12],
        _ => [0, 0, 0, 0, 0, 0, 0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn ticks_use_screen_origin_and_image_scale() {
        let x = Axis::new(125.0, 300.0, 600).unwrap();
        let y = Axis::new(80.0, 250.0, 500).unwrap();
        assert_eq!(x.position(400.0), 550.0);
        assert_eq!(y.position(300.0), 440.0);
        assert_eq!(
            x.ticks().collect::<Vec<_>>(),
            vec![(200.0, 150.0), (300.0, 350.0), (400.0, 550.0)]
        );
        assert_eq!(
            Axis::new(-125.5, 240.0, 480)
                .unwrap()
                .ticks()
                .collect::<Vec<_>>(),
            vec![(-100.0, 51.0), (0.0, 251.0), (100.0, 451.0)]
        );
        assert_eq!(
            Axis::new(0.0, 200.0, 200)
                .unwrap()
                .ticks()
                .collect::<Vec<_>>(),
            vec![(0.0, 0.0), (100.0, 100.0)]
        );
        assert_eq!(Axis::new(80.0, 100.0, 300).unwrap().position(100.0), 60.0);
    }

    #[test]
    fn invalid_geometry_is_error() {
        for (origin, extent, pixels) in [
            (f64::NAN, 100.0, 200),
            (0.0, 0.0, 1),
            (0.0, -1.0, 1),
            (0.0, 100.0, 0),
            (0.0, f64::INFINITY, 1),
        ] {
            assert!(Axis::new(origin, extent, pixels).is_err());
        }
        assert_eq!(number(-125.5), "-125.5");
        assert_eq!(number(-0.001), "0");
        assert_eq!(number(125.567), "125.57");
    }

    #[test]
    fn grid_preserves_dimensions_and_marks_labels() {
        let mut img = RgbaImage::from_pixel(600, 500, image::Rgba([255, 0, 0, 255]));
        // blue marker away from grid lines/labels
        for y in 40..50 {
            for x in 500..510 {
                img.put_pixel(x, y, image::Rgba([0, 0, 255, 255]));
            }
        }
        let before = img.clone();
        overlay(
            &mut img,
            Bounds {
                x: 125.0,
                y: 80.0,
                width: 300.0,
                height: 250.0,
            },
        )
        .unwrap();
        assert_eq!((img.width(), img.height()), (600, 500));
        assert_ne!(img, before);
        // untouched content pixel preserved
        assert_eq!(*img.get_pixel(500, 45), image::Rgba([0, 0, 255, 255]));
        // grid altered something
        assert_ne!(*img.get_pixel(150, 200), *before.get_pixel(150, 200));
    }
}
