use gfx;
use gfx::handle::ShaderResourceView;
use gfx::texture::{AaMode, Kind, Mipmap};
use image;
use graphics::{ColorFormat, Vec4};

pub struct Loader {}

impl Loader {
    /// Loads a texture from file
    pub fn from_file<F, R>(factory: &mut F, image_file_path: &str) -> Option<ShaderResourceView<R, Vec4>>
    where
        F: gfx::Factory<R>,
        R: gfx::Resources,
    {
        match image::open(image_file_path) {
            Ok(image) => {
                let image = image.flipv().to_rgba();
                let (width, height) = image.dimensions();
                let kind = Kind::D2(width as u16, height as u16, AaMode::Single);

                match factory.create_texture_immutable_u8::<ColorFormat>(kind, Mipmap::Provided, &[&image.into_raw()[..]]) {
                    Ok((_, view)) => Some(view),
                    Err(create_texture_error) => panic!(create_texture_error),
                }
            }
            Err(_) => None,
        }
    }
}
