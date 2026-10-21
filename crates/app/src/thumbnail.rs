//! Convert Moonraker thumbnails (PNG/JPEG from the slicer, SVG from the fake) into Slint images.

use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

pub fn decode(data: &[u8]) -> Option<Image> {
    if looks_like_svg(data) {
        return Image::load_from_svg_data(data).ok();
    }
    let rgba = image::load_from_memory(data).ok()?.into_rgba8();
    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(rgba.as_raw(), rgba.width(), rgba.height());
    Some(Image::from_rgba8(buffer))
}

fn looks_like_svg(data: &[u8]) -> bool {
    let start = &data[..data.len().min(256)];
    let text = String::from_utf8_lossy(start);
    let trimmed = text.trim_start();
    trimmed.starts_with("<svg") || (trimmed.starts_with("<?xml") && text.contains("<svg"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_svg_but_not_png() {
        assert!(looks_like_svg(b"  <svg xmlns='http://www.w3.org/2000/svg'/>"));
        assert!(looks_like_svg(b"<?xml version='1.0'?><svg/>"));
        assert!(!looks_like_svg(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn decodes_png() {
        let mut png = Vec::new();
        image::RgbaImage::from_pixel(2, 3, image::Rgba([255, 0, 0, 255]))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();

        let decoded = decode(&png).expect("png decodes");

        assert_eq!((decoded.size().width, decoded.size().height), (2, 3));
    }
}
