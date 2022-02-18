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

#[macro_use]
mod errors;
mod clipboard;
mod dib_section;
mod font_loader;
mod resources;
mod wide_strings;

use crate::dib_section::DIBSection;
use crate::errors::*;
use crate::resources::*;
use crate::wide_strings::WideStringManager;
use application::clipboard::*;
use application::draw_context::*;
use application::font::*;
use application::gui::GuiSystem;
use application::job_system::*;
use application::keys::Key;

pub trait Application {
    fn on_create(&mut self, context: &mut Context);
    fn on_draw(&self, draw_context: &mut DrawContext);
}

pub struct SystemContext {
    application: Box<dyn Application>,
    buffer: Option<DIBSection>,
    context: Context,
}

pub struct Context {
    pub font_factory: FontFactory,
    pub clipboard: Rc<RefCell<dyn Clipboard>>,
    pub job_system: Rc<RefCell<JobSystem>>,
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
        _ => return None,
    }
}

unsafe fn maybe_window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> APIResult<LRESULT> {
    let get_context =
        || -> APIResult<(&mut dyn Application, &mut Option<DIBSection>, &mut Context)> {
            let system_context: &mut SystemContext =
                std::mem::transmute(run_api!(GetWindowLongPtrW(hwnd, GWL_USERDATA))?);
            Ok((
                system_context.application.deref_mut(),
                &mut system_context.buffer,
                &mut system_context.context,
            ))
        };

    fn adjust_window_size(context: &mut Context, hwnd: HWND) -> APIResult<()> {
        let minimal_size = context.gui_system.get_minimal_size();
        let client_rect = get_client_rect(hwnd)?;
        let dx = max(0, minimal_size.0 - (client_rect.right - client_rect.left));
        let dy = max(0, minimal_size.1 - (client_rect.bottom - client_rect.top));
        let mut window_rect = get_window_rect(hwnd)?;
        window_rect.left -= dx / 2;
        window_rect.top -= dy / 2;
        window_rect.right += dx / 2;
        window_rect.bottom += dy / 2;
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

    fn run_jobs(context: &mut Context, hwnd: HWND) -> APIResult<()> {
        context.job_system.borrow_mut().run_all();
        adjust_window_size(context, hwnd)?;
        Ok(())
    }

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
                if context.gui_system.on_char(c) {
                    run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
                }
            }
        }

        WM_KEYDOWN => {
            if let Some(key) = wparam_to_key(wparam) {
                let (_, _, context) = get_context()?;
                if context.gui_system.on_key_down(key) {
                    run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
                }
                run_jobs(context, hwnd)?;
            }
        }

        WM_KEYUP => {
            if let Some(key) = wparam_to_key(wparam) {
                let (_, _, context) = get_context()?;
                if context.gui_system.on_key_up(key) {
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
                context.gui_system.on_resize();
            }

            let buffer = buffer.as_mut().unwrap();
            let mut draw_context = DrawContext {
                buffer: buffer.as_view_mut(),
                font_factory: &mut context.font_factory,
            };

            application.on_draw(&mut draw_context);
            context.gui_system.on_draw(&mut draw_context);
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
            let minimal_size = context.gui_system.get_minimal_size();
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

        WM_CREATE => {
            let create_struct: &mut CREATESTRUCTW = std::mem::transmute(lparam);
            let system_context: &mut SystemContext =
                std::mem::transmute(create_struct.lpCreateParams);
            adjust_window_size(&mut system_context.context, hwnd)?;
        }

        WM_LBUTTONDOWN => {
            run_api!(SetCapture(hwnd))?;
            let position = (
                LOWORD(lparam as u32) as i16 as i32,
                HIWORD(lparam as u32) as i16 as i32,
            );
            let (_, _, context) = get_context()?;
            if context.gui_system.on_mouse_down(position) {
                run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
            }
            run_jobs(context, hwnd)?;
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
            if context.gui_system.on_mouse_wheel(position, delta) {
                run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
            }
        }

        WM_MOUSEMOVE => {
            let position = (
                LOWORD(lparam as u32) as i16 as i32,
                HIWORD(lparam as u32) as i16 as i32,
            );
            let (_, _, context) = get_context()?;
            if context.gui_system.on_mouse_move(position) {
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
            if context.gui_system.on_mouse_leave() {
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
            if context.gui_system.on_mouse_up(position) {
                run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
            }
            run_jobs(context, hwnd)?;
        }

        WM_ACTIVATE => {
            if wparam == WA_INACTIVE as WPARAM {
                let (_, _, context) = get_context()?;
                if context.gui_system.on_deactivate() {
                    run_api!(InvalidateRect(hwnd, 0 as *const RECT, 0))?;
                }
            }
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

fn create_window(name: &str, context: *mut SystemContext) -> APIResult<HWND> {
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
            0,                                // dwExStyle
            wide_strings.from_str(name),      // class we registered
            wide_strings.from_str(name),      // title
            WS_OVERLAPPEDWINDOW | WS_VISIBLE, // dwStyle
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

pub fn run_application(name: &str, application: Box<dyn Application>) -> APIResult<()> {
    std::panic::set_hook(Box::new(|info| {
        use backtrace::Backtrace;
        let bt = Backtrace::new();
        println!("{:?}", info);
        println!("{:?}", bt);
        println!("exit(3)");
        std::process::exit(3);
    }));

    let font_factory = FontFactory::new(Rc::new(RefCell::new(font_loader::GDIFontLoader {})));
    let clipboard = Rc::new(RefCell::new(crate::clipboard::Clipboard::new()));
    let job_system = Rc::new(RefCell::new(JobSystem::new()));
    let gui_system = GuiSystem::new(job_system.clone());

    let mut system_context = SystemContext {
        application,
        buffer: None,
        context: Context {
            clipboard,
            job_system,
            gui_system,
            font_factory,
        },
    };

    system_context
        .application
        .on_create(&mut system_context.context);
    let _window = create_window(name, &mut system_context)?;
    loop {
        if !handle_message()? {
            break;
        }
    }

    Ok(())
}
