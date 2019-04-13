use std::ops::Range;
use gfx_hal::image as i;
use image;

pub const RGBA_IMAGE_STRIDE : usize = 4;

pub struct Loader;

pub struct Texture {
    pub image: image::RgbaImage,
    pub kind: i::Kind,
}

impl Texture {
    pub fn get_image(&self) -> &image::RgbaImage {
        &self.image
    }

    pub fn get_kind(&self) -> &i::Kind {
        &self.kind
    }

    pub fn get_dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    pub fn row_pitch(&self, row_alignment_mask: u32) -> u32 {
        let (width, _) = self.get_dimensions();
        (width * RGBA_IMAGE_STRIDE as u32 + row_alignment_mask) & !row_alignment_mask
    }

    /// Returns a range which denotes the start and finish of bytes
    /// that belong to the row provided by row_index
    pub fn row_range(&self, row_index: usize) -> Range<usize> {
        let (width, _) = self.get_dimensions();
        let width = width as usize;

        let row_index_start = row_index * width * RGBA_IMAGE_STRIDE;
        let row_index_end = (row_index + 1) * width * RGBA_IMAGE_STRIDE;

        row_index_start..row_index_end
    }

    /// Returns the complete size of the image
    pub fn get_upload_size(&self, row_alignment_mask: u32) -> u64 {
        let (_, height) = self.get_dimensions();
        let row_pitch = self.row_pitch(row_alignment_mask);
        (height * row_pitch) as u64
    }
}

impl Loader {
    /// Loads a texture from file
    pub fn from_file(image_file_path: &str) -> Option<Texture> {
        match image::open(image_file_path) {
            Ok(image) => {
                let image = image.to_rgba();
                let (width, height) = image.dimensions();
                let kind = i::Kind::D2(width, height, 1, 1);

                Some(Texture { image, kind })
            }
            Err(_) => None,
        }
    }
}
