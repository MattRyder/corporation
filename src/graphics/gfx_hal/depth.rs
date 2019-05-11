use gfx_hal::*;

struct DepthImage<B: Backend> {
    image: Option<B::Image>,
    image_view: Option<B::ImageView>
}