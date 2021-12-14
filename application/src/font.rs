use crate::image::*;
use std::collections::HashMap;
use std::cmp::max;
use std::cell::RefCell;
use std::rc::Rc;

pub enum Glyph {
  NoAA (Image<bool>),
  AA (Image<u8>),
  TT (Image<u32>),
}

pub struct FontLine {
  name: String,
  size: i32,
  chars: HashMap<char, Glyph>,
}

impl FontLine {
  pub fn new(name: String, size: i32) -> Self {
    Self {
      name, size, chars: Default::default(),
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
  fn load_glyphs(&mut self, font_name: &str, font_size: i32, code_from: u32, code_to: u32) -> HashMap<char, Glyph>;
}

#[derive(Clone)]
pub struct Font {
  color: u32,
  layout_horizontal: TextLayoutHorizontal,
  layout_vertical: TextLayoutVertical,
  line: Rc<RefCell<FontLine>>,
  loader: Rc<RefCell<dyn FontLoader + 'static>>
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
  TOP, MIDDLE, BOTTOM
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TextLayoutHorizontal {
  LEFT, MIDDLE, RIGHT
}

impl Font {
  fn new(
    color: u32,
    layout_horizontal: TextLayoutHorizontal,
    layout_vertical: TextLayoutVertical,
    line: Rc<RefCell<FontLine>>,
    loader: Rc<RefCell<dyn FontLoader + 'static>>
  ) -> Self {
    Self {
      color,
      layout_horizontal,
      layout_vertical,
      line,
      loader
    }
  }

  pub fn color(&self, color: u32) -> Self {
    let mut result = self.clone();
    result.color = color;
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
      let additional_glyphs = self.loader.borrow_mut().load_glyphs(&line.name, line.size, code & !0xFF, (code & !0xFF) + 0x100);
      line.chars.extend(additional_glyphs);
    }

    line.chars.get(&c)
  }

  fn get_size_with(&self, text: &str, line: &mut FontLine) -> ImageSize {
    let mut result = (0, 0);
    for c in text.chars() {
      match self.get_char(c, line) {
        Some(Glyph::NoAA(img)) => {
          let sz = img.get_size();
          result.0 += sz.0;
          result.1 = max(result.1, sz.1);
        },
        _ => {}
      }
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
      }
    );

    let color = self.color;

    for c in text.chars() {
      match self.get_char(c, line) {
        Some(Glyph::NoAA(img)) => {
          dst.draw(img.as_view(), position, |dst, src| if *src {*dst = color;});
          position.0 += img.get_size().0 as i32;
        },
        _ => {}
      }
    }
  }
}

pub struct FontFactory {
  library: HashMap<(String, i32), Rc<RefCell<FontLine>>>,
  loader: Rc<RefCell<dyn FontLoader>>,
}

impl FontFactory {
  pub fn new(loader: Rc<RefCell<dyn FontLoader>>) -> Self {
    Self {
      library: Default::default(),
      loader,
    }
  }

  pub fn new_font(
    &mut self,
    name: &str,
    size: i32,
  ) -> Font {
    let library = self
      .library
      .entry((name.to_string(), size))
      .or_insert_with(|| Rc::new(RefCell::new(FontLine::new(name.to_string(), size))));
    Font::new(0x000000, TextLayoutHorizontal::LEFT, TextLayoutVertical::TOP, library.clone(), self.loader.clone())
  }
}
