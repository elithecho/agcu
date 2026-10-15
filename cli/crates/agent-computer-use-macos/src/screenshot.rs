use agent_computer_use_core::{Error, Result};
use core_graphics::base::{kCGBitmapByteOrder32Big, kCGImageAlphaPremultipliedLast};
use core_graphics::color_space::CGColorSpace;
use core_graphics::context::CGContext;
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use core_graphics::image::CGImage;
use foreign_types::ForeignType;

const SPACING: f64 = 100.0;

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

/// Reject unreadable native captures before annotation can hide their emptiness.
///
/// # Arguments
/// * `image` - A native capture with an arbitrary source pixel layout.
///
/// # Returns
/// Success for visible content, or a capture/normalization error.
pub(crate) fn ensure_readable(image: &CGImage) -> Result<()> {
    let mut normalized = bitmap(
        image.width(),
        image.height(),
        &CGColorSpace::create_device_rgb(),
    )?;
    let area = rect(0.0, 0.0, image.width() as f64, image.height() as f64);
    normalized.clear_rect(area);
    normalized.draw_image(area, image);
    if normalized.data().chunks_exact(4).any(|rgba| rgba[3] != 0) {
        Ok(())
    } else {
        Err(platform_error(
            "capture is wholly transparent; the window may be missing or unreadable",
        ))
    }
}

/// Compose absolute screen-coordinate axes without changing image dimensions.
///
/// # Arguments
/// * `image` - Captured RGB image at its original resolution.
/// * `bounds` - The same capture's bounds in absolute screen points.
///
/// # Returns
/// An annotated image, or a geometry/allocation error before any file write.
pub(crate) fn overlay(image: &CGImage, bounds: CGRect) -> Result<CGImage> {
    let x = Axis::new(bounds.origin.x, bounds.size.width, image.width())?;
    let y = Axis::new(bounds.origin.y, bounds.size.height, image.height())?;
    let context = bitmap(image.width(), image.height(), &image.color_space())?;
    context.draw_image(
        rect(0.0, 0.0, image.width() as f64, image.height() as f64),
        image,
    );
    context.translate(0.0, image.height() as f64);
    context.scale(1.0, -1.0);
    context.set_should_antialias(false);

    for (_, position) in x.ticks() {
        line(
            &context,
            rect(position.floor(), 0.0, 1.0, image.height() as f64),
        );
    }
    for (_, position) in y.ticks() {
        line(
            &context,
            rect(0.0, position.floor(), image.width() as f64, 1.0),
        );
    }

    let unit = (2.0 * x.scale().min(y.scale())).max(1.0).round();
    let badge_height = 9.0 * unit;
    let mut right = label(&context, x.origin, 2.0, 2.0, unit);
    for (coordinate, position) in x.ticks() {
        let start = position.floor() + 3.0;
        if start >= right + 4.0 {
            right = label(&context, coordinate, start, 2.0, unit);
        }
    }
    let mut bottom = 2.0 * badge_height + 4.0;
    label(&context, y.origin, 2.0, badge_height + 4.0, unit);
    for (coordinate, position) in y.ticks() {
        let start = position.floor() + 3.0;
        if start >= bottom + 4.0 {
            label(&context, coordinate, 2.0, start, unit);
            bottom = start + badge_height;
        }
    }
    context
        .create_image()
        .ok_or_else(|| platform_error("failed to compose screenshot grid"))
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> CGRect {
    CGRect::new(&CGPoint::new(x, y), &CGSize::new(width, height))
}

fn line(context: &CGContext, area: CGRect) {
    context.set_rgb_fill_color(0.0, 0.0, 0.0, 0.25);
    context.fill_rect(area.inset(&CGSize::new(-1.0, -1.0)));
    context.set_rgb_fill_color(1.0, 1.0, 1.0, 0.55);
    context.fill_rect(area);
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

fn label(context: &CGContext, coordinate: f64, x: f64, y: f64, unit: f64) -> f64 {
    let text = number(coordinate);
    let width = (text.len() as f64 * 6.0 + 1.0) * unit;
    context.set_rgb_fill_color(0.0, 0.0, 0.0, 0.85);
    context.fill_rect(rect(x, y, width, 9.0 * unit));
    context.set_rgb_fill_color(1.0, 1.0, 1.0, 1.0);
    for (index, digit) in text.bytes().enumerate() {
        for (row, bits) in glyph(digit).into_iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    context.fill_rect(rect(
                        x + (index as f64 * 6.0 + column as f64 + 1.0) * unit,
                        y + (row as f64 + 1.0) * unit,
                        unit,
                        unit,
                    ));
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
        _ => unreachable!("coordinate formatting produces only digits, minus and decimal point"),
    }
}

fn bitmap(width: usize, height: usize, space: &CGColorSpace) -> Result<CGContext> {
    let row = width
        .checked_mul(4)
        .ok_or_else(|| platform_error("screenshot bitmap size overflow"))?;
    let bytes = row
        .checked_mul(height)
        .ok_or_else(|| platform_error("screenshot bitmap size overflow"))?;
    if width == 0 || height == 0 || bytes > isize::MAX as usize {
        return Err(platform_error("invalid screenshot bitmap dimensions"));
    }
    let pointer = unsafe {
        CGBitmapContextCreate(
            std::ptr::null_mut(),
            width,
            height,
            8,
            row,
            space.as_ptr(),
            kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big,
        )
    };
    if pointer.is_null() {
        return Err(platform_error("failed to allocate screenshot bitmap"));
    }
    Ok(unsafe { CGContext::from_ptr(pointer) })
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGBitmapContextCreate(
        data: *mut std::ffi::c_void,
        width: usize,
        height: usize,
        bits_per_component: usize,
        bytes_per_row: usize,
        space: core_graphics::sys::CGColorSpaceRef,
        bitmap_info: u32,
    ) -> core_graphics::sys::CGContextRef;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_bitmap(width: usize, height: usize) -> Result<CGContext> {
        bitmap(width, height, &CGColorSpace::create_device_rgb())
    }

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
    fn invalid_geometry_and_allocation_are_errors() {
        for (origin, extent, pixels) in [
            (f64::NAN, 100.0, 200),
            (0.0, 0.0, 1),
            (0.0, -1.0, 1),
            (0.0, 100.0, 0),
            (0.0, f64::INFINITY, 1),
        ] {
            assert!(matches!(
                Axis::new(origin, extent, pixels),
                Err(Error::PlatformError { .. })
            ));
        }
        assert!(matches!(
            fixture_bitmap(usize::MAX, 2),
            Err(Error::PlatformError { .. })
        ));
        assert!(matches!(
            fixture_bitmap(0, 2),
            Err(Error::PlatformError { .. })
        ));
        assert_eq!(number(-125.5), "-125.5");
        assert_eq!(number(-0.001), "0");
        assert_eq!(number(125.567), "125.57");
    }

    #[test]
    fn grid_preserves_content_dimensions_orientation_and_clips() {
        let context = fixture_bitmap(600, 500).unwrap();
        context.set_rgb_fill_color(1.0, 0.0, 0.0, 1.0);
        context.fill_rect(rect(0.0, 0.0, 600.0, 500.0));
        context.set_rgb_fill_color(0.0, 0.0, 1.0, 1.0);
        context.fill_rect(rect(500.0, 40.0, 10.0, 10.0));
        let image = context.create_image().unwrap();
        let output = overlay(&image, rect(125.0, 80.0, 300.0, 250.0)).unwrap();
        assert_eq!((output.width(), output.height()), (600, 500));
        let pixel = |image: &CGImage, x: usize, y: usize| {
            let data = image.data();
            let offset = y * image.bytes_per_row() + x * 4;
            data.bytes()[offset..offset + 4].to_vec()
        };
        assert_eq!(pixel(&output, 500, 455), vec![0, 0, 255, 255]);
        assert_eq!(pixel(&output, 90, 200), vec![255, 0, 0, 255]);
        assert_eq!(pixel(&output, 150, 200), vec![226, 140, 140, 255]);
        assert_eq!(pixel(&output, 2, 2), vec![38, 0, 0, 255]);
        let negative = overlay(&image, rect(-125.5, 80.0, 300.0, 250.0)).unwrap();
        assert_eq!(pixel(&negative, 6, 18), vec![255, 255, 255, 255]);
        assert_eq!(pixel(&negative, 106, 26), vec![255, 255, 255, 255]);
        let tiny = fixture_bitmap(8, 8).unwrap().create_image().unwrap();
        let output = overlay(&tiny, rect(-125.5, -80.0, 4.0, 4.0)).unwrap();
        assert_eq!((output.width(), output.height()), (8, 8));
    }

    #[test]
    fn grid_keeps_rgb_profile_and_uncovered_pixel_values() {
        use core_graphics::color_space::{kCGColorSpaceDisplayP3, kCGColorSpaceSRGB};
        for name in unsafe { [kCGColorSpaceSRGB, kCGColorSpaceDisplayP3] } {
            let space = CGColorSpace::create_with_name(name).unwrap();
            let mut context = bitmap(400, 400, &space).unwrap();
            for pixel in context.data().chunks_exact_mut(4) {
                pixel.copy_from_slice(&[123, 67, 89, 255]);
            }
            let image = context.create_image().unwrap();
            let output = overlay(&image, rect(125.0, 80.0, 200.0, 200.0)).unwrap();
            let data = output.data();
            let offset = 200 * output.bytes_per_row() + 90 * 4;
            assert_eq!(&data.bytes()[offset..offset + 4], &[123, 67, 89, 255]);
        }
    }
}
