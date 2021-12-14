
use crate::APIResult;
use crate::APIResultCode;
use crate::OpenedClipboard;
use crate::GlobalLockedPointer;

use winapi::um::winuser::*;
use winapi::shared::ntdef::*;
use std::os::windows::ffi::OsStringExt;

pub struct Clipboard {
}

unsafe fn wstr_len(ptr: *const u16) -> usize {
  let mut result = 0;
  while *ptr.add(result) != 0 {
    result += 1;
  }

  result
}

impl Clipboard {
  pub fn new() -> Self {
    Self {}
  }

  fn maybe_get_string(&self) -> APIResult<Option<String>> {
    unsafe {
      let _clipboard = OpenedClipboard::new()?; // use for RAII
      let hdata = GetClipboardData(CF_UNICODETEXT);
      if hdata == 0 as HANDLE {
        return Ok(None);
      }

      let locked_pointer = GlobalLockedPointer::new(hdata)?;
      let u16_pointer: *const u16 = std::mem::transmute(locked_pointer.get_data());
      if u16_pointer == 0 as *const u16 {
        return Ok(None);
      }
      let u16str = std::slice::from_raw_parts(u16_pointer, wstr_len(u16_pointer));
      let str = std::ffi::OsString::from_wide(u16str).into_string().map_err(|e| {
        println!("{:?}", e);
        APIResultCode::new(0x203D)
      })?;
      Ok(Some(str))
    }
  }

  fn maybe_put_string(&mut self, _text: &str) -> APIResult<()> {
    unimplemented!()
  }
}

impl application::clipboard::Clipboard for Clipboard {
  fn get_string(&self) -> Option<String> {
    self.maybe_get_string().unwrap_or_default()
  }

  fn put_string(&mut self, text: &str) {
    self.maybe_put_string(text).unwrap_or_default()
  }
}

impl std::fmt::Debug for Clipboard {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.pad("Windows clipboard")
  }
}

