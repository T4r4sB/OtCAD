use crate::image::*;
use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FontAntiAliasingMode {
    NoAA,
    AA,
    TT,
}

impl Default for FontAntiAliasingMode {
    fn default() -> Self {
        FontAntiAliasingMode::NoAA
    }
}

pub enum Glyph {
    NoAA(Image<bool>),
    AA(Image<u8>, Image<u8>),
    TT(Image<u32>, Image<u32>),
}

pub struct FontLine {
    name: String,
    size: i32,
    anti_aliasing_mode: FontAntiAliasingMode,
    chars: HashMap<char, Glyph>,
}

impl FontLine {
    pub fn new(name: String, size: i32, anti_aliasing_mode: FontAntiAliasingMode) -> Self {
        Self {
            name,
            size,
            anti_aliasing_mode,
            chars: Default::default(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_size(&self) -> i32 {
        self.size
    }
}

pub trait FontLoader {
    fn load_glyphs(
        &mut self,
        font_name: &str,
        font_size: i32,
        code_from: u32,
        code_to: u32,
        anti_aliasing_mode: FontAntiAliasingMode,
    ) -> HashMap<char, Glyph>;
}

#[derive(Clone)]
pub struct Font {
    color: u32,
    light: bool,
    layout_horizontal: TextLayoutHorizontal,
    layout_vertical: TextLayoutVertical,
    line: Rc<RefCell<FontLine>>,
    loader: Rc<RefCell<dyn FontLoader + 'static>>,
}

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("name", &self.get_info().get_name())
            .field("size", &self.get_info().get_size())
            .field("color", &self.color)
            .field("layout_horizontal", &self.layout_horizontal)
            .field("layout_vertical", &self.layout_vertical)
            .finish()
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TextLayoutVertical {
    TOP,
    MIDDLE,
    BOTTOM,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TextLayoutHorizontal {
    LEFT,
    MIDDLE,
    RIGHT,
}

impl Font {
    fn new(
        color: u32,
        layout_horizontal: TextLayoutHorizontal,
        layout_vertical: TextLayoutVertical,
        line: Rc<RefCell<FontLine>>,
        loader: Rc<RefCell<dyn FontLoader + 'static>>,
    ) -> Self {
        Self {
            color,
            light: Self::color_is_light(color),
            layout_horizontal,
            layout_vertical,
            line,
            loader,
        }
    }

    fn color_is_light(color: u32) -> bool {
        let r = color & 0xFF;
        let g = (color >> 8) & 0xFF;
        let b = (color >> 16) & 0xFF;
        r + g + b >= 0x180
    }

    pub fn color(&self, color: u32) -> Self {
        let mut result = self.clone();
        result.color = color;
        result.light = Self::color_is_light(color);
        result
    }

    pub fn layout_vertical(&self, layout: TextLayoutVertical) -> Self {
        let mut result = self.clone();
        result.layout_vertical = layout;
        result
    }

    pub fn layout_horizontal(&self, layout: TextLayoutHorizontal) -> Self {
        let mut result = self.clone();
        result.layout_horizontal = layout;
        result
    }

    pub fn get_info(&self) -> std::cell::Ref<FontLine> {
        self.line.borrow()
    }

    fn get_char<'i>(&self, c: char, line: &'i mut FontLine) -> Option<&'i Glyph> {
        if !line.chars.contains_key(&c) {
            let code = c as u32;
            let additional_glyphs = self.loader.borrow_mut().load_glyphs(
                &line.name,
                line.size,
                code & !0xFF,
                (code & !0xFF) + 0x100,
                line.anti_aliasing_mode,
            );
            line.chars.extend(additional_glyphs);
        }

        line.chars.get(&c)
    }

    fn get_size_with(&self, text: &str, line: &mut FontLine) -> ImageSize {
        let mut result = (0, 0);
        for c in text.chars() {
            let sz = match self.get_char(c, line) {
                Some(Glyph::NoAA(img)) => img.get_size(),
                Some(Glyph::AA(img, _)) => img.get_size(),
                Some(Glyph::TT(img, _)) => img.get_size(),
                None => (0, 0),
            };
            result.0 += sz.0;
            result.1 = max(result.1, sz.1);
        }

        result
    }

    pub fn get_size(&self, text: &str) -> ImageSize {
        let line = &mut self.line.borrow_mut();
        self.get_size_with(text, line)
    }

    pub fn draw(&self, text: &str, position: Position, dst: &mut ImageViewMut<u32>) {
        let line = &mut self.line.borrow_mut();
        let size = self.get_size_with(text, line);
        let mut position = (
            match self.layout_horizontal {
                TextLayoutHorizontal::LEFT => position.0,
                TextLayoutHorizontal::MIDDLE => position.0 - size.0 as i32 / 2,
                TextLayoutHorizontal::RIGHT => position.0 - size.0 as i32,
            },
            match self.layout_vertical {
                TextLayoutVertical::TOP => position.1,
                TextLayoutVertical::MIDDLE => position.1 - size.1 as i32 / 2,
                TextLayoutVertical::BOTTOM => position.1 - size.1 as i32,
            },
        );

        let color = self.color;

        for c in text.chars() {
            match self.get_char(c, line) {
                Some(Glyph::NoAA(img)) => {
                    dst.draw(&img.as_view(), position, |dst, src| {
                        if *src {
                            *dst = color;
                        }
                    });
                    position.0 += img.get_size().0 as i32;
                }
                Some(Glyph::AA(img_black, img_white)) => {
                    let img = if self.light { &img_white } else { &img_black };
                    dst.draw(&img.as_view(), position, |dst, src| {
                        let dst_r = (*dst & 0xFF) as i32;
                        let dst_g = ((*dst >> 8) & 0xFF) as i32;
                        let dst_b = ((*dst >> 16) & 0xFF) as i32;
                        let dst_a = (*dst >> 24) as i32;

                        let color_r = (color & 0xFF) as i32;
                        let color_g = ((color >> 8) & 0xFF) as i32;
                        let color_b = ((color >> 16) & 0xFF) as i32;
                        let color_a = (color >> 24) as i32;

                        let result_r =
                            (dst_r + (((color_r - dst_r) * (*src as i32) + 255) >> 8)) as u32;
                        let result_g =
                            (dst_g + (((color_g - dst_g) * (*src as i32) + 255) >> 8)) as u32;
                        let result_b =
                            (dst_b + (((color_b - dst_b) * (*src as i32) + 255) >> 8)) as u32;
                        let result_a =
                            (dst_a + (((color_a - dst_a) * (*src as i32) + 255) >> 8)) as u32;
                        *dst = result_r | (result_g << 8) | (result_b << 16) | (result_a << 24);
                    });
                    position.0 += img.get_size().0 as i32;
                }
                Some(Glyph::TT(img_black, img_white)) => {
                    let img = if self.light { &img_white } else { &img_black };
                    dst.draw(&img.as_view(), position, |dst, src| {
                        let dst_r = (*dst & 0xFF) as i32;
                        let dst_g = ((*dst >> 8) & 0xFF) as i32;
                        let dst_b = ((*dst >> 16) & 0xFF) as i32;
                        let dst_a = (*dst >> 24) as i32;

                        let src_r = (*src & 0xFF) as i32;
                        let src_g = ((*src >> 8) & 0xFF) as i32;
                        let src_b = ((*src >> 16) & 0xFF) as i32;
                        let src_a = (*src >> 24) as i32;

                        let color_r = (color & 0xFF) as i32;
                        let color_g = ((color >> 8) & 0xFF) as i32;
                        let color_b = ((color >> 16) & 0xFF) as i32;
                        let color_a = (color >> 24) as i32;

                        let result_r =
                            (dst_r + (((color_r - dst_r) * (src_r as i32) + 255) >> 8)) as u32;
                        let result_g =
                            (dst_g + (((color_g - dst_g) * (src_g as i32) + 255) >> 8)) as u32;
                        let result_b =
                            (dst_b + (((color_b - dst_b) * (src_b as i32) + 255) >> 8)) as u32;
                        let result_a =
                            (dst_a + (((color_a - dst_a) * (src_a as i32) + 255) >> 8)) as u32;
                        *dst = result_r | (result_g << 8) | (result_b << 16) | (result_a << 24);
                    });
                    position.0 += img.get_size().0 as i32;
                }
                None => {}
            }
        }
    }
}

pub struct FontFactory {
    library: HashMap<(String, i32, FontAntiAliasingMode), Rc<RefCell<FontLine>>>,
    loader: Rc<RefCell<dyn FontLoader>>,
}

impl FontFactory {
    pub fn new(loader: impl FontLoader + 'static) -> Self {
        Self {
            library: Default::default(),
            loader: Rc::new(RefCell::new(loader)),
        }
    }

    pub fn new_font(
        &mut self,
        name: &str,
        size: i32,
        anti_aliasing_mode: FontAntiAliasingMode,
    ) -> Font {
        let library_line = self
            .library
            .entry((name.to_string(), size, anti_aliasing_mode))
            .or_insert_with(|| {
                Rc::new(RefCell::new(FontLine::new(
                    name.to_string(),
                    size,
                    anti_aliasing_mode,
                )))
            });
        Font::new(
            0x000000,
            TextLayoutHorizontal::LEFT,
            TextLayoutVertical::TOP,
            library_line.clone(),
            self.loader.clone(),
        )
    }
}
