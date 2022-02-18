use std::mem::MaybeUninit;

use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::errhandlingapi::*;
use winapi::um::winbase::*;
use winapi::um::wingdi::*;
use winapi::um::winuser::*;

use crate::APIResult;
use crate::APIResultCode;

pub struct PaintStructContext {
    paint_struct: PAINTSTRUCT,
    hwnd: HWND,
    dc: HDC,
}

impl PaintStructContext {
    pub unsafe fn new(hwnd: HWND) -> APIResult<Self> {
        let mut paint_struct = MaybeUninit::uninit();
        let dc = run_api!(BeginPaint(hwnd, paint_struct.as_mut_ptr()))?;
        Ok(Self {
            paint_struct: paint_struct.assume_init(),
            hwnd,
            dc,
        })
    }

    pub unsafe fn get_dc(&self) -> HDC {
        self.dc
    }
}

impl Drop for PaintStructContext {
    fn drop(&mut self) {
        unsafe {
            if let Err(_) = run_api!(EndPaint(self.hwnd, &mut self.paint_struct)) {
                // We cant pass this error anywhere, because it is a destructor
            }
        }
    }
}

pub struct AutoHGDIObj {
    handle: HGDIOBJ,
}

impl AutoHGDIObj {
    pub fn new(handle: HGDIOBJ) -> Self {
        Self { handle }
    }

    pub fn get_handle(&self) -> HGDIOBJ {
        self.handle
    }
}

impl Drop for AutoHGDIObj {
    fn drop(&mut self) {
        unsafe {
            if let Err(_) = run_api!(DeleteObject(self.handle)) {
                // We cant pass this error anywhere, because it is a destructor
            }
        }
    }
}

pub struct GotHDC {
    hwnd: HWND,
    dc: HDC,
}

impl GotHDC {
    pub unsafe fn new(hwnd: HWND) -> APIResult<Self> {
        Ok(Self {
            hwnd,
            dc: run_api!(GetDC(hwnd))?,
        })
    }

    pub fn get_dc(&self) -> HDC {
        self.dc
    }
}

impl Drop for GotHDC {
    fn drop(&mut self) {
        unsafe {
            if let Err(_) = run_api!(ReleaseDC(self.hwnd, self.dc)) {
                // We cant pass this error anywhere, because it is a destructor
            }
        }
    }
}

pub struct AutoHDC {
    dc: HDC,
}

impl AutoHDC {
    pub fn new(dc: HDC) -> Self {
        Self { dc }
    }

    pub fn get_dc(&self) -> HDC {
        self.dc
    }
}

impl Drop for AutoHDC {
    fn drop(&mut self) {
        unsafe {
            if let Err(_) = run_api!(DeleteDC(self.dc)) {
                // We cant pass this error anywhere, because it is a destructor
            }
        }
    }
}

pub struct OpenedClipboard {}

impl OpenedClipboard {
    pub fn new() -> APIResult<Self> {
        unsafe {
            run_api!(OpenClipboard(0 as HWND))?;
            Ok(Self {})
        }
    }
}

impl Drop for OpenedClipboard {
    fn drop(&mut self) {
        unsafe {
            if let Err(_) = run_api!(CloseClipboard()) {
                // We cant pass this error anywhere, because it is a destructor
            }
        }
    }
}

pub struct GlobalLockedPointer {
    hmem: HGLOBAL,
    data: LPVOID,
}

impl GlobalLockedPointer {
    pub fn new(hmem: HGLOBAL) -> APIResult<Self> {
        unsafe {
            Ok(Self {
                hmem,
                data: run_api!(GlobalLock(hmem))?,
            })
        }
    }

    pub fn get_data(&self) -> LPVOID {
        self.data
    }
}

impl Drop for GlobalLockedPointer {
    fn drop(&mut self) {
        unsafe {
            if let Err(_) = run_api!(GlobalUnlock(self.hmem)) {
                // We cant pass this error anywhere, because it is a destructor
            }
        }
    }
}
