use application::image::*;
use application::font::*;
use std::collections::HashMap;
use std::mem::MaybeUninit;

use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::errhandlingapi::*;
use winapi::um::wingdi::*;
use winapi::um::winuser::*;

use crate::DIBSection;
use crate::WideStringManager;

use crate::APIResult;
use crate::APIResultCode;

use crate::AutoHGDIObj;

pub struct GDIFontLoader {
}

impl GDIFontLoader {
  fn maybe_load_glyphs(&mut self, font_name: &str, font_size: i32, code_from: u32, code_to: u32) -> APIResult<HashMap<char, Glyph>> {
    let mut dst = DIBSection::new((1,1))?;
    let mut max_size = dst.get_size();
    let mut wide_strings = WideStringManager::new();
    let mut result = HashMap::new();

    unsafe {
      let font = AutoHGDIObj::new(run_api!(CreateFontW(
        font_size, 0, 0, 0,
        FW_NORMAL,
        FALSE as u32, FALSE as u32, FALSE as u32,
        OEM_CHARSET, OUT_RASTER_PRECIS,
        CLIP_DEFAULT_PRECIS, NONANTIALIASED_QUALITY, DEFAULT_PITCH | FF_DONTCARE, wide_strings.from_str(font_name)))? as HGDIOBJ);

      let mut dc = dst.get_dc();
      SelectObject(dc, font.get_handle());

      for code in code_from .. code_to {
        if let Some(c) = char::from_u32(code) {
          let s = c.to_string();
          let ws = wide_strings.from_str(&s);
          let mut rect = MaybeUninit::uninit();
          run_api!(GetTextExtentPoint32W(dc, ws, 1, rect.as_mut_ptr()))?;
          let rect = rect.assume_init();
          let rect = (rect.cx as usize, rect.cy as usize);

          max_size = (std::cmp::max(rect.0, max_size.0), std::cmp::max(rect.1, max_size.1));

          if dst.get_size() != max_size {
            dst = DIBSection::new(max_size)?;
            dc = dst.get_dc();
            SelectObject(dc, font.get_handle());
          }

          let mut text_rect = RECT{left: 0, top: 0, right: max_size.0 as i32, bottom: max_size.1 as i32};
          dst.as_view_mut().fill(|p| *p = 0xFFFFFF);
          run_api!(DrawTextW(dc, ws, 1, &mut text_rect, DT_SINGLELINE | DT_LEFT | DT_TOP))?;
          let mut new_image = Image::new(rect);
          new_image.as_view_mut().draw(dst.as_view(), (0, 0), |d, s| *d = *s == 0);
          result.insert(c, Glyph::NoAA(new_image));
        }
      }
    }

    Ok(result)
  }
}

impl FontLoader for GDIFontLoader {
  fn load_glyphs(&mut self, font_name: &str, font_size: i32, code_from: u32, code_to: u32) -> HashMap<char, Glyph> {
    self.maybe_load_glyphs(font_name, font_size, code_from, code_to).unwrap_or_default()
  }
}