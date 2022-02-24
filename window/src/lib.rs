use std::cell::RefCell;
use std::cmp::{max, min};
use std::mem::MaybeUninit;
use std::ops::DerefMut;
use std::os::windows::ffi::OsStringExt;
use std::rc::Rc;

use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::errhandlingapi::*;
use winapi::um::libloaderapi::*;
use winapi::um::wingdi::*;
use winapi::um::winuser::*;

use crate::dib_section::DIBSection;
use crate::errors::*;
use crate::resources::*;
use crate::wide_strings::WideStringManager;
use application::clipboard::*;
use application::draw_context::*;
use application::font::*;
use application::gui::GuiSystem;
use application::image::*;
use application::job_system::*;
use application::keys::Key;
use serde::{Deserialize, Serialize};

#[macro_use]
mod errors;
mod clipboard;
mod dib_section;
mod font_loader;
mod resources;
mod wide_strings;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct WindowPosition {
    pub maximized: bool,
    pub left_top: Position,
    pub right_bottom: Position,
}

pub trait Application {
    fn on_create(&mut self, context: Rc<RefCell<Context>>);
    fn on_close(&mut self, context: Rc<RefCell<Context>>);
    fn on_change_position(&mut self, window_position: WindowPosition);
    fn on_draw(&self, draw_context: &mut DrawContext);
}

pub struct SystemContext {
    application: Box<dyn Application>,
    buffer: Option<DIBSection>,
    context: Rc<RefCell<Context>>,
}

pub struct Context {
    hwnd: HWND,
    pub font_factory: FontFactory,
    pub clipboard: Clipboard,
    pub job_system: JobSystem,
    pub gui_system: GuiSystem,
}

pub fn get_client_rect(hwnd: HWND) -> APIResult<RECT> {
    unsafe {
        let mut rect = MaybeUninit::uninit();
        run_api!(GetClientRect(hwnd, rect.as_mut_ptr()))?;
        Ok(rect.assume_init())
    }
}

pub fn get_window_rect(hwnd: HWND) -> APIResult<RECT> {
    unsafe {
        let mut rect = MaybeUninit::uninit();
        run_api!(GetWindowRect(hwnd, rect.as_mut_ptr()))?;
        Ok(rect.assume_init())
    }
}

pub fn get_desctop_rect() -> APIResult<RECT> {
    unsafe {
        let mut rect = MaybeUninit::<RECT>::uninit();
        run_api!(SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            std::mem::transmute(rect.as_mut_ptr()),
            0
        ))?;
        Ok(rect.assume_init())
    }
}

pub fn adjust_rect(src: RECT, dst: &mut RECT) {
    if dst.left < src.left {
        dst.right += src.left - dst.left;
        dst.left = src.left;
    }

    if dst.top < src.top {
        dst.bottom += src.top - dst.top;
        dst.top = src.top;
    }

    if dst.right > src.right {
        dst.left -= dst.right - src.right;
        dst.right = src.right;
    }

    if dst.bottom > src.bottom {
        dst.top -= dst.bottom - src.bottom;
        dst.bottom = src.bottom;
    }

    dst.left = max(dst.left, src.left);
    dst.top = max(dst.top, src.top);
    dst.right = min(dst.right, src.right);
    dst.bottom = min(dst.bottom, src.bottom);
}

fn wparam_to_key(code: WPARAM) -> Option<Key> {
    match code as i32 {
        VK_LEFT => return Some(Key::Left),
        VK_RIGHT => return Some(Key::Right),
        VK_UP => return Some(Key::Up),
        VK_DOWN => return Some(Key::Down),
        VK_BACK => return Some(Key::Back),
        VK_INSERT => return Some(Key::Insert),
        VK_DELETE => return Some(Key::Delete),
        VK_SPACE => return Some(Key::Space),
        VK_NUMPAD0 => return Some(Key::Numpad0),
        VK_NUMPAD1 => return Some(Key::Numpad1),
        VK_NUMPAD2 => return Some(Key::Numpad2),
        VK_NUMPAD3 => return Some(Key::Numpad3),
        VK_NUMPAD4 => return Some(Key::Numpad4),
        VK_NUMPAD5 => return Some(Key::Numpad5),
        VK_NUMPAD6 => return Some(Key::Numpad6),
        VK_NUMPAD7 => return Some(Key::Numpad7),
        VK_NUMPAD8 => return Some(Key::Numpad8),
        VK_NUMPAD9 => return Some(Key::Numpad9),
        VK_ESCAPE => return Some(Key::Escape),
        VK_RETURN => return Some(Key::Enter),
        _ => return None,
    }
}

fn adjust_window_size(context: Rc<RefCell<Context>>, hwnd: HWND) -> APIResult<()> {
    let minimal_size = context.borrow().gui_system.get_minimal_size();
    let client_rect = get_client_rect(hwnd)?;
    if client_rect.right - client_rect.left >= minimal_size.0
        && client_rect.bottom - client_rect.top >= minimal_size.1
    {
        return Ok(());
    }

    let dx = max(0, minimal_size.0 - (client_rect.right - client_rect.left));
    let dy = max(0, minimal_size.1 - (client_rect.bottom - client_rect.top));
    let mut window_rect = get_window_rect(hwnd)?;
    window_rect.left -= dx / 2;
    window_rect.top -= dy / 2;
    window_rect.right += dx / 2;
    window_rect.bottom += dy / 2;
    adjust_rect(get_desctop_rect()?, &mut window_rect);

    unsafe {
        run_api!(SetWindowPos(
            hwnd,
            0 as HWND,
            window_rect.left,
            window_rect.top,
            window_rect.right - window_rect.left,
            window_rect.bottom - window_rect.top,
            0
        ))?;
    }

    Ok(())
}

fn run_jobs(context: Rc<RefCell<Context>>, hwnd: HWND) -> APIResult<()> {
    let job_system = context.borrow_mut().job_system.clone();
    job_system.run_all(); // jobs can borrow context
    adjust_window_size(context, hwnd)?;
    Ok(())
}

fn get_window_position(hwnd: HWND) -> APIResult<WindowPosition> {
    unsafe {
        let mut wp = MaybeUninit::<WINDOWPLACEMENT>::zeroed().assume_init();
        wp.length = std::mem::size_of::<WINDOWPLACEMENT>() as u32;
        run_api!(GetWindowPlacement(hwnd, &mut wp))?;
        let rect = get_window_rect(hwnd)?;
        let maximized = wp.showCmd as i32 == SW_SHOWMAXIMIZED;
        Ok(WindowPosition {
            maximized,
            left_top: (rect.left, rect.top),
            right_bottom: (rect.right, rect.bottom),
        })
    }
}

unsafe fn maybe_window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> APIResult<LRESULT> {
    let get_context =
        || -> APIResult<(&mut dyn Application, &mut Option<DIBSection>, Rc<RefCell<Context>>)> {
            let system_context: &mut SystemContext =
                std::mem::transmute(run_api!(GetWindowLongPtrW(hwnd, GWL_USERDATA))?);
            Ok((
                system_context.application.deref_mut(),
                &mut system_context.buffer,
                system_context.context.clone(),
            ))
        };

    match msg {
        WM_CHAR => {
            let codes = [wparam as u16];
            let str = std::ffi::OsString::from_wide(&codes)
                .into_string()
                .map_err(|_| {
                    APIResultCode::new(0x203D) // ERROR_DS_DECODING_ERROR
                })?;

            let (_, _, context) = get_context()?;
            for c in str.chars() {
                if context.borrow_mut().gui_system.on_char(c) {
                    run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
                }
            }
        }

        WM_KEYDOWN => {
            if let Some(key) = wparam_to_key(wparam) {
                let (_, _, context) = get_context()?;
                if context.borrow_mut().gui_system.on_key_down(key) {
                    run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
                }
                run_jobs(context.clone(), hwnd)?;
            }
        }

        WM_KEYUP => {
            if let Some(key) = wparam_to_key(wparam) {
                let (_, _, context) = get_context()?;
                if context.borrow_mut().gui_system.on_key_up(key) {
                    run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
                }
            }
        }

        WM_SYSKEYDOWN => {}

        WM_SYSCHAR => {
            return Ok(0);
        }

        WM_SYSKEYUP => {}

        WM_PAINT => {
            let rect = get_client_rect(hwnd)?;
            let rect_size = (
                (rect.right - rect.left) as usize,
                (rect.bottom - rect.top) as usize,
            );
            let (application, buffer, context) = get_context()?;
            if buffer.is_none() || buffer.as_ref().unwrap().get_size() != rect_size {
                *buffer = Some(DIBSection::new(rect_size)?);
                context.borrow_mut().gui_system.on_resize();
            }

            let buffer = buffer.as_mut().unwrap();
            let mut context_borrow_mut = context.borrow_mut();
            let context_ref = context_borrow_mut.deref_mut();
            let mut draw_context = DrawContext {
                buffer: buffer.as_view_mut(),
                font_factory: &mut context_ref.font_factory,
            };

            application.on_draw(&mut draw_context);
            context_ref.gui_system.on_draw(&mut draw_context);
            let paint_struct_context = PaintStructContext::new(hwnd)?;
            run_api!(BitBlt(
                paint_struct_context.get_dc(),
                0,
                0,
                rect_size.0 as i32,
                rect_size.1 as i32,
                buffer.get_dc(),
                0,
                0,
                SRCCOPY
            ))?;
        }

        WM_SIZING => {
            let p_rect: *mut RECT = std::mem::transmute(lparam);
            let rect = &mut *p_rect;

            let client_rect = get_client_rect(hwnd)?;
            let window_rect = get_window_rect(hwnd)?;

            let (_, _, context) = get_context()?;
            let minimal_size = context.borrow().gui_system.get_minimal_size();
            // Count size of bevel...
            let minimal_size_x = minimal_size.0 as i32 + (window_rect.right - window_rect.left)
                - (client_rect.right - client_rect.left);
            let minimal_size_y = minimal_size.1 as i32 + (window_rect.bottom - window_rect.top)
                - (client_rect.bottom - client_rect.top);

            match wparam as u32 {
                WMSZ_BOTTOMRIGHT | WMSZ_RIGHT | WMSZ_TOPRIGHT => {
                    rect.right = max(rect.right, rect.left + minimal_size_x)
                }
                WMSZ_BOTTOMLEFT | WMSZ_LEFT | WMSZ_TOPLEFT => {
                    rect.left = min(rect.left, rect.right - minimal_size_x)
                }
                WMSZ_BOTTOM | WMSZ_TOP => {}
                _ => eprintln!("Wrong wparam {} in WM_SIZING message!", wparam),
            }

            match wparam as u32 {
                WMSZ_BOTTOM | WMSZ_BOTTOMLEFT | WMSZ_BOTTOMRIGHT => {
                    rect.bottom = max(rect.bottom, rect.top + minimal_size_y)
                }
                WMSZ_TOP | WMSZ_TOPLEFT | WMSZ_TOPRIGHT => {
                    rect.top = min(rect.top, rect.bottom - minimal_size_y)
                }
                WMSZ_LEFT | WMSZ_RIGHT => {}
                _ => eprintln!("Wrong wparam {} in WM_SIZING message!", wparam),
            }
        }

        WM_SIZE => {
            let (application, _, context) = get_context()?;
            adjust_window_size(context.clone(), hwnd)?;
            application.on_change_position(get_window_position(hwnd)?);
        }

        WM_MOVING => {
            let p_rect: *mut RECT = std::mem::transmute(lparam);
            let rect = &mut *p_rect;
            adjust_rect(get_desctop_rect()?, rect);
        }

        WM_MOVE => {
            let (application, _, _) = get_context()?;
            application.on_change_position(get_window_position(hwnd)?);
        }

        WM_LBUTTONDOWN => {
            run_api!(SetCapture(hwnd))?;
            let position = (
                LOWORD(lparam as u32) as i16 as i32,
                HIWORD(lparam as u32) as i16 as i32,
            );
            let (_, _, context) = get_context()?;
            if context.borrow_mut().gui_system.on_mouse_down(position) {
                run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
            }
            run_jobs(context.clone(), hwnd)?;
        }

        WM_MOUSEWHEEL => {
            let mut point = POINT {
                x: LOWORD(lparam as u32) as i16 as i32,
                y: HIWORD(lparam as u32) as i16 as i32,
            };

            run_api!(ScreenToClient(hwnd, &mut point))?;
            let position = (point.x, point.y);
            let delta = -(HIWORD(wparam as u32) as i16 as i32) / (WHEEL_DELTA as i32);
            let (_, _, context) = get_context()?;
            if context
                .borrow_mut()
                .gui_system
                .on_mouse_wheel(position, delta)
            {
                run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
            }
        }

        WM_MOUSEMOVE => {
            let position = (
                LOWORD(lparam as u32) as i16 as i32,
                HIWORD(lparam as u32) as i16 as i32,
            );
            let (_, _, context) = get_context()?;
            if context.borrow_mut().gui_system.on_mouse_move(position) {
                run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
            }

            let mut tme = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                hwndTrack: hwnd,
                dwFlags: TME_HOVER | TME_LEAVE,
                dwHoverTime: HOVER_DEFAULT,
            };
            run_api!(TrackMouseEvent(&mut tme))?;
        }

        WM_MOUSELEAVE => {
            let (_, _, context) = get_context()?;
            if context.borrow_mut().gui_system.on_mouse_leave() {
                run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
            }
        }

        WM_LBUTTONUP => {
            run_api!(ReleaseCapture())?;
            let position = (
                LOWORD(lparam as u32) as i16 as i32,
                HIWORD(lparam as u32) as i16 as i32,
            );
            let (_, _, context) = get_context()?;
            if context.borrow_mut().gui_system.on_mouse_up(position) {
                run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
            }
            run_jobs(context.clone(), hwnd)?;
        }

        WM_ACTIVATE => {
            if wparam == WA_INACTIVE as WPARAM {
                let (_, _, context) = get_context()?;
                if context.borrow_mut().gui_system.on_deactivate() {
                    run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
                }
            }
        }

        WM_CLOSE => {
            let (application, _, context) = get_context()?;
            application.on_close(context.clone());
        }

        WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => {}
    }

    Ok(DefWindowProcW(hwnd, msg, wparam, lparam))
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match maybe_window_proc(hwnd, msg, wparam, lparam) {
        Ok(l_result) => return l_result,
        Err(_) => {
            // Do nothing, read message and continue
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
    }
}

fn create_window(
    name: &str,
    context: *mut SystemContext,
    window_position: Option<WindowPosition>,
) -> APIResult<HWND> {
    let mut wide_strings = WideStringManager::new();

    unsafe {
        let hinstance = run_api!(GetModuleHandleW(0 as *const u16))?;
        let wnd_class = WNDCLASSW {
            style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance,
            lpszClassName: wide_strings.from_str(name),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: 0 as HICON,
            hCursor: run_api!(LoadCursorW(0 as HINSTANCE, IDC_ARROW))?,
            hbrBackground: 0 as HBRUSH,
            lpszMenuName: 0 as *const u16,
        };

        run_api!(RegisterClassW(&wnd_class))?;
        let hwnd = run_api!(CreateWindowExW(
            0,                           // dwExStyle
            wide_strings.from_str(name), // class we registered
            wide_strings.from_str(name), // title
            WS_OVERLAPPEDWINDOW,         // dwStyle
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,                // size and position
            0 as HWND,                    // hWndParent
            0 as HMENU,                   // hMenu
            hinstance,                    // hInstance
            std::mem::transmute(context)  // lpParam
        ))?;

        run_api!(SetWindowLongPtrW(
            hwnd,
            GWL_USERDATA,
            std::mem::transmute(context)
        ))?;

        if let Some(window_position) = window_position {
            run_api!(SetWindowPos(
                hwnd,
                0 as HWND,
                window_position.left_top.0,
                window_position.left_top.1,
                window_position.right_bottom.0 - window_position.left_top.0,
                window_position.right_bottom.1 - window_position.left_top.1,
                0
            ))?;

            if window_position.maximized {
                run_api!(ShowWindow(hwnd, SW_SHOWMAXIMIZED))?;
            } else {
                run_api!(ShowWindow(hwnd, SW_SHOWDEFAULT))?;
            }
        } else {
            run_api!(ShowWindow(hwnd, SW_SHOWDEFAULT))?;
        }

        Ok(hwnd)
    }
}

fn handle_message() -> APIResult<bool> {
    unsafe {
        let mut msg = MaybeUninit::<MSG>::uninit();
        if run_api!(GetMessageW(msg.as_mut_ptr(), 0 as HWND, 0, 0))? > 0 {
            // skip errors
            let _ = run_api!(TranslateMessage(msg.as_ptr()));
            let _ = run_api!(DispatchMessageW(msg.as_ptr()));
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

pub fn show_message(context: Rc<RefCell<Context>>, text: &str, caption: &str) {
    let mut wide_strings = WideStringManager::new();
    let hwnd = context.borrow().hwnd;
    unsafe {
        MessageBoxW(
            hwnd,
            wide_strings.from_str(text),
            wide_strings.from_str(caption),
            MB_OK,
        );
    }
}

pub fn get_screen_resolution() -> ImageSize {
    unsafe {
        (
            GetSystemMetrics(SM_CXSCREEN) as usize,
            GetSystemMetrics(SM_CYSCREEN) as usize,
        )
    }
}

pub fn run_application(
    name: &str,
    application: Box<dyn Application>,
    window_position: Option<WindowPosition>,
) -> APIResult<()> {
    std::panic::set_hook(Box::new(|info| {
        use backtrace::Backtrace;
        let bt = Backtrace::new();
        println!("{:#?}", info);
        println!("{:?}", bt);
        println!("exit(3)");
        std::process::exit(3);
    }));

    let font_factory = FontFactory::new(font_loader::GDIFontLoader {});
    let clipboard = application::clipboard::Clipboard::new(crate::clipboard::Clipboard::new());
    let job_system = JobSystem::new();
    let gui_system = GuiSystem::new(job_system.clone());

    let mut system_context = SystemContext {
        application,
        buffer: None,
        context: Rc::new(RefCell::new(Context {
            hwnd: 0 as HWND,
            clipboard,
            job_system,
            gui_system,
            font_factory,
        })),
    };

    system_context
        .application
        .on_create(system_context.context.clone());
    let hwnd = create_window(name, &mut system_context, window_position)?;
    system_context.context.borrow_mut().hwnd = hwnd;
    loop {
        if !handle_message()? {
            break;
        }
    }

    Ok(())
}
