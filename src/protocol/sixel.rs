use std::io;

use icy_sixel::{BackgroundMode, EncodeOptions, PixelAspectRatio, SixelImage};
use image::imageops::FilterType;
use ratatui::crossterm::terminal::{window_size, WindowSize};

/// Sixel uses pixels rather than terminal cells, so obtain the actual font size.
/// Refuse an unknown size instead of producing oversized or misaligned rows.
pub(crate) fn cell_size() -> io::Result<(u16, u16)> {
    cell_size_from(window_size()?)
}

fn cell_size_from(size: WindowSize) -> io::Result<(u16, u16)> {
    let width = size.width.checked_div(size.columns).unwrap_or(0);
    let height = size.height.checked_div(size.rows).unwrap_or(0);
    if width == 0 || height == 0 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Sixel requires a terminal that reports its size in pixels; run Serie directly in the graphics-enabled Alacritty",
        ));
    }
    Ok((width, height))
}

pub(super) fn encode(png: &[u8], columns: usize, clear_columns: usize) -> crate::Result<String> {
    encode_at_size(png, columns, clear_columns, cell_size()?)
}

fn encode_at_size(
    png: &[u8],
    columns: usize,
    clear_columns: usize,
    cell: (u16, u16),
) -> crate::Result<String> {
    if clear_columns < columns {
        return Err("Sixel clear width must cover the graph image".into());
    }
    let width = u32::try_from(columns)?
        .checked_mul(u32::from(cell.0))
        .filter(|width| *width > 0)
        .ok_or("invalid Sixel graph width")?;
    let height = u32::from(cell.1);
    let image = image::load_from_memory(png)?
        .resize_exact(width, height, FilterType::Lanczos3)
        .to_rgba8();
    let sixel = SixelImage::try_from_rgba(image.into_raw(), width as usize, height as usize)?
        .with_aspect_ratio(PixelAspectRatio::Square)
        .with_background_mode(BackgroundMode::Transparent)
        .encode_with(&EncodeOptions {
            max_colors: 64,
            diffusion: 0.0,
            ..Default::default()
        })?;

    // Transparent Sixel pixels leave existing graphics untouched. Erase the
    // entire graph row before replacing it, including columns outside this
    // compact image that may contain pixels from a previously wider row.
    // ECH uses the current background and leaves the cursor in place, preserving
    // the terminal's transparent background without erasing the commit text.
    // Restore the cursor after Sixel to match Ratatui's ForcedWidth(1) cell.
    Ok(format!("\x1b7\x1b[{clear_columns}X{sixel}\x1b8\x1b[1C"))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn cell_dimensions_use_reported_terminal_pixels() {
        let size = WindowSize {
            columns: 80,
            rows: 24,
            width: 800,
            height: 576,
        };
        assert_eq!(cell_size_from(size).unwrap(), (10, 24));
    }

    #[test]
    fn missing_pixel_dimensions_are_rejected() {
        for (columns, rows, width, height) in [
            (80, 24, 0, 0),
            (0, 24, 800, 576),
            (80, 0, 800, 576),
            (80, 24, 800, 0),
        ] {
            let error = cell_size_from(WindowSize {
                columns,
                rows,
                width,
                height,
            })
            .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::Unsupported);
        }
    }

    #[test]
    fn graph_row_is_scaled_to_cells_and_restores_the_cursor() {
        let image = image::RgbaImage::from_fn(50, 50, |x, _| {
            if x < 25 {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([0, 0, 0, 0])
            }
        });
        let mut png = Cursor::new(Vec::new());
        image.write_to(&mut png, image::ImageFormat::Png).unwrap();
        let encoded = encode_at_size(png.get_ref(), 2, 2, (10, 24)).unwrap();
        assert!(encoded.starts_with("\x1b7\x1b[2X\x1bP"));
        assert!(encoded.ends_with("\x1b\\\x1b8\x1b[1C"));
        let payload = encoded
            .strip_prefix("\x1b7\x1b[2X")
            .unwrap()
            .strip_suffix("\x1b8\x1b[1C")
            .unwrap();
        let decoded: SixelImage = payload.parse().unwrap();
        assert_eq!(decoded.dimensions(), (20, 24));
        assert!(decoded.has_transparency());
    }

    #[test]
    fn odd_font_height_does_not_add_a_visible_sixel_band() {
        let image = image::RgbaImage::from_pixel(50, 50, image::Rgba([0, 255, 0, 255]));
        let mut png = Cursor::new(Vec::new());
        image.write_to(&mut png, image::ImageFormat::Png).unwrap();
        let encoded = encode_at_size(png.get_ref(), 2, 2, (16, 35)).unwrap();
        let payload = encoded
            .strip_prefix("\x1b7\x1b[2X")
            .unwrap()
            .strip_suffix("\x1b8\x1b[1C")
            .unwrap();
        let decoded: SixelImage = payload.parse().unwrap();
        assert_eq!(decoded.width, 32);
        assert!(decoded.height >= 35);
        assert!(decoded.pixels[32 * 35 * 4..]
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| pixel[3] == 0));
    }

    #[test]
    fn compact_replacement_erases_the_previous_full_graph_footprint() {
        let image = image::RgbaImage::from_pixel(25, 50, image::Rgba([255, 0, 0, 255]));
        let mut png = Cursor::new(Vec::new());
        image.write_to(&mut png, image::ImageFormat::Png).unwrap();

        // A one-lane row can replace a multi-lane row at the same screen
        // position when scrolling. The clear must cover all eight columns,
        // even though the compact image occupies only two of them.
        let encoded = encode_at_size(png.get_ref(), 2, 8, (10, 24)).unwrap();
        let payload = encoded
            .strip_prefix("\x1b7\x1b[8X")
            .expect("the whole previous graph row must be erased before drawing")
            .strip_suffix("\x1b8\x1b[1C")
            .unwrap();
        let decoded: SixelImage = payload.parse().unwrap();
        assert_eq!(decoded.dimensions(), (20, 24));
        assert!(encode_at_size(png.get_ref(), 2, 1, (10, 24)).is_err());
    }
}
