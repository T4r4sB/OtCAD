use application::font::*;
use application::image::*;
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

pub struct GDIFontLoader {}

impl GDIFontLoader {
    fn maybe_load_glyphs(
        &mut self,
        font_name: &str,
        font_size: i32,
        code_from: u32,
        code_to: u32,
        anti_aliasing_mode: FontAntiAliasingMode,
    ) -> APIResult<HashMap<char, Glyph>> {
        let mut dst = DIBSection::new((1, 1))?;
        let mut max_size = dst.get_size();
        let mut wide_strings = WideStringManager::new();
        let mut result = HashMap::new();

        unsafe {
            let quality = match anti_aliasing_mode {
                FontAntiAliasingMode::NoAA => NONANTIALIASED_QUALITY,
                FontAntiAliasingMode::AA => ANTIALIASED_QUALITY,
                FontAntiAliasingMode::TT => CLEARTYPE_QUALITY,
            };

            let font = AutoHGDIObj::new(run_api!(CreateFontW(
                font_size,
                0,
                0,
                0,
                FW_NORMAL,
                FALSE as u32,
                FALSE as u32,
                FALSE as u32,
                OEM_CHARSET,
                OUT_RASTER_PRECIS,
                CLIP_DEFAULT_PRECIS,
                quality,
                DEFAULT_PITCH | FF_DONTCARE,
                wide_strings.from_str(font_name)
            ))? as HGDIOBJ);

            let mut dc = dst.get_dc();
            SelectObject(dc, font.get_handle());

            for code in code_from..code_to {
                if let Some(c) = char::from_u32(code) {
                    let s = c.to_string();
                    let ws = wide_strings.from_str(&s);
                    let mut rect = MaybeUninit::uninit();
                    run_api!(GetTextExtentPoint32W(dc, ws, 1, rect.as_mut_ptr()))?;
                    let rect = rect.assume_init();
                    let rect = (rect.cx as usize, rect.cy as usize);

                    max_size = (
                        std::cmp::max(rect.0, max_size.0),
                        std::cmp::max(rect.1, max_size.1),
                    );

                    if dst.get_size() != max_size {
                        dst = DIBSection::new(max_size)?;
                        dc = dst.get_dc();
                        SelectObject(dc, font.get_handle());
                    }

                    let mut text_rect = RECT {
                        left: 0,
                        top: 0,
                        right: max_size.0 as i32,
                        bottom: max_size.1 as i32,
                    };

                    macro_rules! draw_text {
                        () => {
                            run_api!(DrawTextW(
                                dc,
                                ws,
                                1,
                                &mut text_rect,
                                DT_SINGLELINE | DT_LEFT | DT_TOP
                            ))?;
                        };
                    }

                    macro_rules! draw_black {
                        () => {
                            dst.as_view_mut().fill(|p| *p = 0xFFFFFF);
                            SetTextColor(dc, 0x000000);
                            SetBkColor(dc, 0xFFFFFF);
                            draw_text!();
                        };
                    }

                    macro_rules! draw_white {
                        () => {
                            dst.as_view_mut().fill(|p| *p = 0x000000);
                            SetTextColor(dc, 0xFFFFFF);
                            SetBkColor(dc, 0x000000);
                            draw_text!();
                        };
                    }

                    match anti_aliasing_mode {
                        FontAntiAliasingMode::NoAA => {
                            draw_black!();
                            let mut new_image = Image::new(rect);
                            new_image
                                .as_view_mut()
                                .draw(dst.as_view(), (0, 0), |d, s| *d = *s == 0);
                            result.insert(c, Glyph::NoAA(new_image));
                        }
                        FontAntiAliasingMode::AA => {
                            draw_black!();
                            let mut new_image_dark = Image::new(rect);
                            new_image_dark
                                .as_view_mut()
                                .draw(dst.as_view(), (0, 0), |d, s| *d = 0xFF - (*s & 0xFF) as u8);
                            draw_white!();
                            let mut new_image_light = Image::new(rect);
                            new_image_light
                                .as_view_mut()
                                .draw(dst.as_view(), (0, 0), |d, s| *d = (*s & 0xFF) as u8);
                            result.insert(c, Glyph::AA(new_image_dark, new_image_light));
                        }
                        FontAntiAliasingMode::TT => {
                            draw_black!();
                            let mut new_image_dark = Image::new(rect);
                            new_image_dark
                                .as_view_mut()
                                .draw(dst.as_view(), (0, 0), |d, s| *d = !*s);
                            draw_white!();
                            let mut new_image_light = Image::new(rect);
                            new_image_light
                                .as_view_mut()
                                .draw(dst.as_view(), (0, 0), |d, s| *d = *s);
                            result.insert(c, Glyph::TT(new_image_dark, new_image_light));
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}

impl FontLoader for GDIFontLoader {
    fn load_glyphs(
        &mut self,
        font_name: &str,
        font_size: i32,
        code_from: u32,
        code_to: u32,
        anti_aliasing_mode: FontAntiAliasingMode,
    ) -> HashMap<char, Glyph> {
        let mut anti_aliasing_mode = anti_aliasing_mode;
        loop {
            let result = self.maybe_load_glyphs(
                font_name,
                font_size,
                code_from,
                code_to,
                anti_aliasing_mode,
            );
            if result.is_ok() || anti_aliasing_mode == FontAntiAliasingMode::NoAA {
                return result.unwrap_or_default();
            }

            if anti_aliasing_mode == FontAntiAliasingMode::AA {
                anti_aliasing_mode = FontAntiAliasingMode::NoAA;
            } else if anti_aliasing_mode == FontAntiAliasingMode::TT {
                anti_aliasing_mode = FontAntiAliasingMode::AA;
            }
        }
    }
}
