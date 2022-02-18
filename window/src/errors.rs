use std::os::windows::ffi::OsStringExt;

use winapi::shared::minwindef::*;
use winapi::shared::ntdef::*;
use winapi::um::errhandlingapi::*;
use winapi::um::winbase::*;

#[derive(Default, Copy, Clone)]
pub struct APIResultCode {
    code: DWORD,
}

pub type APIResult<T> = Result<T, APIResultCode>;

pub fn code_to_string(code: DWORD) -> String {
    unsafe {
        if code == 0 {
            return "".to_string();
        }

        let mut message_buffer = NULL as *mut u16;
        let size = FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER
                | FORMAT_MESSAGE_FROM_SYSTEM
                | FORMAT_MESSAGE_IGNORE_INSERTS,
            NULL,
            code,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as u32,
            std::mem::transmute(&mut message_buffer),
            0,
            0 as *mut *mut i8,
        );

        if size == 0 {
            LocalFree(message_buffer as HLOCAL);
            let new_code = GetLastError();
            return format!(
                "Failed with code {}, can not get info because of code {}",
                code, new_code
            );
        }

        let maybe_str = std::ffi::OsString::from_wide(std::slice::from_raw_parts(
            message_buffer,
            size as usize,
        ))
        .into_string();
        LocalFree(message_buffer as HLOCAL);
        match maybe_str {
            Ok(str) => return str.to_string(),
            Err(error) => {
                return format!(
                    "Failed recode error message with code {}, because of {:?}",
                    code, error
                )
            }
        }
    }
}

impl APIResultCode {
    pub fn new(code: DWORD) -> Self {
        Self { code }
    }
}

impl std::fmt::Debug for APIResultCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Code {}: {}", self.code, code_to_string(self.code))
    }
}

macro_rules! run_api_no_skip {
    ($e: expr) => {{
        SetLastError(0);
        let api_result = $e;
        let api_error_code = GetLastError();
        if api_error_code != 0 {
            let code = APIResultCode::new(api_error_code);
            eprintln!("Failed to run {}: {:?}", stringify!($e), code);
            Err(code)
        } else {
            Ok(api_result)
        }
    }};
}

macro_rules! run_api {
    ($e: expr) => {{
        let r = run_api_no_skip!($e);
        SetLastError(0);
        r
    }};
}
