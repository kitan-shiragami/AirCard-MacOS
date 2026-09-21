use std::io::Cursor;
use std::path::Path;

use anyhow::{Context, Result};
use eframe::egui;
use image::{DynamicImage, ImageFormat, imageops::FilterType};

pub const CARD_WIDTH: u32 = 1_536;
pub const CARD_HEIGHT: u32 = 969;

/// Image placement in card coordinates. Zoom 1 fills the card; Fit shows the full image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImagePlacement {
    pub zoom: f32,
    pub offset: egui::Vec2,
}

impl Default for ImagePlacement {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: egui::Vec2::ZERO,
        }
    }
}

impl ImagePlacement {
    pub fn fit_zoom(source: egui::Vec2) -> f32 {
        let ratios = egui::vec2(CARD_WIDTH as f32 / source.x, CARD_HEIGHT as f32 / source.y);
        ratios.x.min(ratios.y) / ratios.x.max(ratios.y)
    }

    pub fn image_rect(&self, source: egui::Vec2) -> egui::Rect {
        let card = egui::vec2(CARD_WIDTH as f32, CARD_HEIGHT as f32);
        let scale = (card.x / source.x).max(card.y / source.y) * self.zoom;
        egui::Rect::from_center_size((card * 0.5 + self.offset * card).to_pos2(), source * scale)
    }

    pub fn constrain(&mut self, source: egui::Vec2) {
        self.zoom = self.zoom.clamp(Self::fit_zoom(source), 8.0);
        let size = self.image_rect(source).size();
        // Oversized images stay over the card; smaller images can move within it.
        let limit = egui::vec2(
            (size.x / CARD_WIDTH as f32 - 1.0).abs() * 0.5,
            (size.y / CARD_HEIGHT as f32 - 1.0).abs() * 0.5,
        );
        self.offset.x = self.offset.x.clamp(-limit.x, limit.x);
        self.offset.y = self.offset.y.clamp(-limit.y, limit.y);
    }
}

pub struct SkinEditor {
    source: image::RgbaImage,
    pub placement: ImagePlacement,
}

impl SkinEditor {
    pub fn from_path(path: &Path) -> Result<Self> {
        let source =
            image::open(path).with_context(|| format!("Could not decode {}", path.display()))?;
        Self::from_image(source)
    }

    pub fn from_image(image: DynamicImage) -> Result<Self> {
        anyhow::ensure!(image.width() > 0 && image.height() > 0, "Image is empty");
        let mut source = image.to_rgba8();
        // One opaque background for the preview, exported PNG, and Wallet PDF.
        for pixel in source.pixels_mut() {
            let alpha = pixel[3] as u16;
            for channel in &mut pixel.0[..3] {
                *channel = ((*channel as u16 * alpha + 127) / 255) as u8;
            }
            pixel[3] = 255;
        }
        Ok(Self {
            source,
            placement: ImagePlacement::default(),
        })
    }

    pub fn source_size(&self) -> egui::Vec2 {
        egui::vec2(self.source.width() as f32, self.source.height() as f32)
    }

    pub fn preview(&self) -> egui::ColorImage {
        // Keep large originals for export without uploading enormous GPU textures.
        let preview = image::imageops::thumbnail(&self.source, 2048, 2048);
        egui::ColorImage::from_rgba_unmultiplied(
            [preview.width() as usize, preview.height() as usize],
            preview.as_raw(),
        )
    }

    pub fn prepare(&self) -> Result<PreparedSkin> {
        let mut placement = self.placement;
        placement.constrain(self.source_size());
        let rect = placement.image_rect(self.source_size());
        // Prefilter when shrinking, then sample only the card canvas. Even extreme
        // panoramas and high zoom never allocate an enlarged full-size image.
        let reduced;
        let source = if rect.width() < self.source.width() as f32 {
            reduced = image::imageops::resize(
                &self.source,
                (rect.width().round() as u32).max(1),
                (rect.height().round() as u32).max(1),
                FilterType::Lanczos3,
            );
            &reduced
        } else {
            &self.source
        };
        let rgba = image::RgbaImage::from_fn(CARD_WIDTH, CARD_HEIGHT, |x, y| {
            let point = egui::pos2(x as f32 + 0.5, y as f32 + 0.5);
            if !rect.contains(point) {
                return image::Rgba([0, 0, 0, 255]);
            }
            let uv = (point - rect.min) / rect.size();
            let sx = (uv.x * source.width() as f32 - 0.5).clamp(0.0, (source.width() - 1) as f32);
            let sy = (uv.y * source.height() as f32 - 0.5).clamp(0.0, (source.height() - 1) as f32);
            image::imageops::interpolate_bilinear(source, sx, sy)
                .unwrap_or(image::Rgba([0, 0, 0, 255]))
        });
        let mut png = Vec::new();
        DynamicImage::ImageRgba8(rgba)
            .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
            .context("Could not encode prepared PNG")?;
        let pdf = png_to_pdf(&png).context("Could not generate card PDF artwork")?;
        Ok(PreparedSkin { png, pdf })
    }
}

pub struct PreparedSkin {
    pub png: Vec<u8>,
    pub pdf: Vec<u8>,
}

pub fn png_to_pdf(png_bytes: &[u8]) -> Result<Vec<u8>> {
    let img =
        image::load_from_memory(png_bytes).context("Failed to decode image for PDF conversion")?;
    let rgb = img.to_rgb8();
    let width = rgb.width();
    let height = rgb.height();
    let raw_bytes = rgb.into_raw();
    let compressed_stream = miniz_oxide::deflate::compress_to_vec_zlib(&raw_bytes, 6);

    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n");

    let mut offsets = Vec::new();

    // 1 0 obj: Catalog
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // 2 0 obj: Pages
    offsets.push(pdf.len());
    pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    // 3 0 obj: Page
    offsets.push(pdf.len());
    let page_obj = format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents 4 0 R /Resources << /XObject << /Im0 5 0 R >> >> >>\nendobj\n",
        width, height
    );
    pdf.extend_from_slice(page_obj.as_bytes());

    // 4 0 obj: Contents stream
    offsets.push(pdf.len());
    let content_stream = format!("q\n{} 0 0 {} 0 0 cm\n/Im0 Do\nQ\n", width, height);
    let contents_obj = format!(
        "4 0 obj\n<< /Length {} >>\nstream\n{}endstream\nendobj\n",
        content_stream.len(),
        content_stream
    );
    pdf.extend_from_slice(contents_obj.as_bytes());

    // 5 0 obj: Image XObject
    offsets.push(pdf.len());
    let image_header = format!(
        "5 0 obj\n<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n",
        width,
        height,
        compressed_stream.len()
    );
    pdf.extend_from_slice(image_header.as_bytes());
    pdf.extend_from_slice(&compressed_stream);
    pdf.extend_from_slice(b"\nendstream\nendobj\n");

    // xref table
    let xref_offset = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", offsets.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for &off in &offsets {
        pdf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }

    // trailer
    let trailer = format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        offsets.len() + 1,
        xref_offset
    );
    pdf.extend_from_slice(trailer.as_bytes());

    Ok(pdf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn striped_image() -> DynamicImage {
        DynamicImage::ImageRgba8(image::RgbaImage::from_fn(120, 40, |x, _| {
            image::Rgba(match x / 40 {
                0 => [255, 0, 0, 255],
                1 => [0, 255, 0, 255],
                _ => [0, 0, 255, 255],
            })
        }))
    }

    #[test]
    fn fitted_export_preserves_entire_image_and_black_margins() {
        let mut editor = SkinEditor::from_image(striped_image()).unwrap();
        editor.placement.zoom = ImagePlacement::fit_zoom(editor.source_size());
        let prepared = editor.prepare().unwrap();
        let png = image::load_from_memory(&prepared.png).unwrap().to_rgba8();
        assert_eq!(png.dimensions(), (CARD_WIDTH, CARD_HEIGHT));
        assert_eq!(png.get_pixel(768, 10).0, [0, 0, 0, 255]);
        assert_eq!(png.get_pixel(100, 484).0, [255, 0, 0, 255]);
        assert_eq!(png.get_pixel(768, 484).0, [0, 255, 0, 255]);
        assert_eq!(png.get_pixel(1436, 484).0, [0, 0, 255, 255]);

        // The PDF must contain the same adjusted pixels as the PNG.
        let image_marker = b"/Subtype /Image";
        let object_start = prepared
            .pdf
            .windows(image_marker.len())
            .position(|w| w == image_marker)
            .unwrap();
        let stream_marker = b"stream\n";
        let stream_start = object_start
            + prepared.pdf[object_start..]
                .windows(stream_marker.len())
                .position(|w| w == stream_marker)
                .unwrap()
            + stream_marker.len();
        let header = std::str::from_utf8(&prepared.pdf[object_start..stream_start]).unwrap();
        let length: usize = header
            .split("/Length ")
            .nth(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let pixels = miniz_oxide::inflate::decompress_to_vec_zlib(
            &prepared.pdf[stream_start..stream_start + length],
        )
        .unwrap();
        assert_eq!(pixels, DynamicImage::ImageRgba8(png).to_rgb8().into_raw());
    }

    #[test]
    fn zoom_and_pan_change_exported_crop() {
        let mut editor = SkinEditor::from_image(striped_image()).unwrap();
        editor.placement.zoom = 2.0;
        editor.placement.offset.x = 100.0;
        editor.placement.constrain(editor.source_size());
        let prepared = editor.prepare().unwrap();
        let png = image::load_from_memory(&prepared.png).unwrap().to_rgba8();
        assert_eq!(png.get_pixel(768, 484).0, [255, 0, 0, 255]);
        // Reset discards both the zoom and position adjustments.
        editor.placement = ImagePlacement::default();
        let prepared = editor.prepare().unwrap();
        let png = image::load_from_memory(&prepared.png).unwrap().to_rgba8();
        assert_eq!(png.get_pixel(768, 484).0, [0, 255, 0, 255]);
    }

    #[test]
    fn placement_bounds_cover_landscape_portrait_and_tiny_sources() {
        for source in [
            egui::vec2(120.0, 40.0),
            egui::vec2(40.0, 120.0),
            egui::vec2(1.0, 1.0),
        ] {
            let mut placement = ImagePlacement {
                zoom: 100.0,
                offset: egui::vec2(100.0, -100.0),
            };
            placement.constrain(source);
            let rect = placement.image_rect(source);
            assert_eq!(placement.zoom, 8.0);
            assert!(rect.min.x <= 0.01 && rect.min.y <= 0.01);
            assert!(
                rect.max.x >= CARD_WIDTH as f32 - 0.01 && rect.max.y >= CARD_HEIGHT as f32 - 0.01
            );
            placement.zoom = 0.0;
            placement.constrain(source);
            let rect = placement.image_rect(source);
            assert!(rect.min.x >= -0.01 && rect.min.y >= -0.01);
            assert!(
                rect.max.x <= CARD_WIDTH as f32 + 0.01 && rect.max.y <= CARD_HEIGHT as f32 + 0.01
            );
        }
    }

    #[test]
    fn transparency_uses_the_same_black_background_as_preview() {
        let image = image::RgbaImage::from_pixel(1, 1, image::Rgba([200, 100, 50, 128]));
        let editor = SkinEditor::from_image(DynamicImage::ImageRgba8(image)).unwrap();
        assert_eq!(editor.source.get_pixel(0, 0).0, [100, 50, 25, 255]);
        assert!(SkinEditor::from_image(DynamicImage::new_rgba8(0, 0)).is_err());
    }

    #[test]
    fn test_png_to_pdf_conversion() {
        let dummy = DynamicImage::new_rgb8(10, 10);
        let mut png = Vec::new();
        dummy
            .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
            .unwrap();

        let pdf = png_to_pdf(&png).expect("png_to_pdf failed");
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n"));
        let pdf_str = String::from_utf8_lossy(&pdf);
        assert!(pdf_str.contains("/FlateDecode"));
        assert!(pdf_str.contains("/MediaBox [0 0 10 10]"));
        assert!(pdf_str.contains("xref"));
    }
}
