use crate::output::{FrameFormat, PixelFormat};
use image::ColorType;

pub fn convert_to_rgba(data: &mut [u8], frame_format: &FrameFormat) -> Option<ColorType> {
	let format = frame_format.format;
	let row_bytes = (frame_format.width * 4) as usize;
	let stride = frame_format.stride as usize;
	let height = frame_format.height as usize;

	if row_bytes == stride {
		// Fast path: tightly packed, process entire buffer
		convert_rows(data, format);
		Some(ColorType::Rgba8)
	} else {
		// Slow path: process row by row, skipping padding
		for y in 0..height {
			let row_start = y * stride;
			convert_rows(&mut data[row_start..row_start + row_bytes], format);
		}
		Some(ColorType::Rgba8)
	}
}

fn convert_rows(data: &mut [u8], format: PixelFormat) {
	match format {
		PixelFormat::Xrgb8888 => {
			for chunk in data.chunks_exact_mut(4) {
				chunk.swap(0, 2);
				chunk[3] = 255;
			}
		}
		PixelFormat::Argb8888 => {
			for chunk in data.chunks_exact_mut(4) {
				chunk.swap(0, 2);
			}
		}
		PixelFormat::Xbgr8888 => {
			for chunk in data.chunks_exact_mut(4) {
				chunk[3] = 255;
			}
		}
		PixelFormat::Abgr8888 => {}
		PixelFormat::Xbgr2101010 => {
			for chunk in data.chunks_exact_mut(4) {
				let pixel = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);

				chunk[0] = ((pixel & 0x3FF) >> 2) as u8;
				chunk[1] = (((pixel >> 10) & 0x3FF) >> 2) as u8;
				chunk[2] = (((pixel >> 20) & 0x3FF) >> 2) as u8;
				chunk[3] = 255;
			}
		}
		PixelFormat::Abgr2101010 => {
			for chunk in data.chunks_exact_mut(4) {
				let pixel = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);

				chunk[0] = ((pixel & 0x3FF) >> 2) as u8;
				chunk[1] = (((pixel >> 10) & 0x3FF) >> 2) as u8;
				chunk[2] = (((pixel >> 20) & 0x3FF) >> 2) as u8;

				let a = (pixel >> 30) & 0x3;
				chunk[3] = (a * 85) as u8;
			}
		}
	}
}
