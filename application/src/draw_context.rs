use crate::font::*;
use crate::image::*;

pub struct DrawContext<'i> {
    pub buffer: ImageViewMut<'i, u32>,
    pub font_factory: &'i mut FontFactory,
}

impl<'i> DrawContext<'i> {
    pub fn window(&mut self, left_top: ImageSize, right_bottom: ImageSize) -> DrawContext {
        DrawContext {
            buffer: self.buffer.window_mut(left_top, right_bottom),
            font_factory: self.font_factory,
        }
    }
}
