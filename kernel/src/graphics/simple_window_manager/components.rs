use crate::{
    error::{Error, Result},
    fs::file::bitmap::BitmapImage,
    graphics::{draw::Draw, multi_layer},
};

use super::RgbColorCode;

pub struct Image {
    pub layer_id: usize,
}

impl Drop for Image {
    fn drop(&mut self) {
        let _ = multi_layer::remove_layer(self.layer_id);
    }
}

impl Image {
    pub fn new(
        bitmap_image: &BitmapImage,
        x: usize,
        y: usize,
        always_on_top: bool,
    ) -> Result<Self> {
        if !bitmap_image.is_valid() {
            return Err(Error::Failed("Invalid bitmap image"));
        }

        let mut layer = multi_layer::create_layer_from_bitmap_image(x, y, bitmap_image)?;
        layer.always_on_top = always_on_top;
        let layer_id = layer.id;
        multi_layer::push_layer(layer)?;
        Ok(Self { layer_id })
    }
}
