pub mod gui_components;

use crate::draw_context::*;
use crate::font::*;
use crate::image::*;
use crate::job_system::*;
use crate::keys::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::{Rc, Weak};

#[derive(Default, Copy, Clone, Debug)]
pub struct SizeConstraint {
    pub absolute: i32,
    pub relative: i32,
}

impl SizeConstraint {
    pub fn fixed(absolute: i32) -> Self {
        Self {
            absolute,
            relative: 0,
        }
    }

    pub fn flexible(absolute: i32) -> Self {
        Self {
            absolute,
            relative: 100,
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct SizeConstraints(pub SizeConstraint, pub SizeConstraint);

#[derive(Default, Copy, Clone, Debug)]
pub struct Rect {
    pub left_top: Position,
    pub right_bottom: Position,
}

impl Rect {
    pub fn contains(self, position: Position) -> bool {
        self.left_top.0 <= position.0
            && self.left_top.1 <= position.1
            && self.right_bottom.0 > position.0
            && self.right_bottom.1 > position.1
    }

    pub fn relative(self, position: Position) -> Position {
        (position.0 - self.left_top.0, position.1 - self.left_top.1)
    }
}

#[derive(Clone)]
pub struct HotkeyCallback(Rc<dyn Fn() + 'static>);

impl std::fmt::Debug for HotkeyCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("HotkeyCallback")
    }
}

#[derive(Debug)]
pub struct GuiControlBase {
    pub(crate) size_constraints: SizeConstraints,
    pub(crate) current_size_constraints: SizeConstraints,
    pub(crate) self_ref: Option<Weak<RefCell<dyn GuiControl>>>,
    pub visible: bool,
    pub(crate) need_redraw: bool,
    pub(crate) focus: bool,
    pub(crate) highlight: bool,
    pub(crate) pressed: bool,
    pub(crate) rect: Rect,
}

impl GuiControlBase {
    pub fn new(size_constraints: SizeConstraints) -> Self {
        let result = Self {
            size_constraints,
            current_size_constraints: size_constraints,
            self_ref: None,
            visible: true,
            need_redraw: false,
            focus: false,
            highlight: false,
            pressed: false,
            rect: Rect::default(),
        };

        result
    }

    pub fn can_draw(&mut self, force: bool) -> bool {
        let result = self.visible && (self.need_redraw || force);
        self.need_redraw = false;
        result
    }

    pub fn erase_background(buffer: &mut ImageViewMut<u32>, theme: &GuiColorTheme) {
        buffer.fill(|p| *p = theme.background);
    }
}

pub enum GuiMessage<'i, 'j> {
    Draw(&'i mut ImageViewMut<'j, u32>, &'i GuiColorTheme, bool),
    UpdateSizeConstraints(&'i mut SizeConstraints),
    FindDestination(&'i mut Rc<RefCell<dyn GuiControl>>, Position),
    RectUpdated,
    FocusLose(JobSystem),
    MouseDown(Position),
    MouseMove(Position),
    MouseUp(Position, JobSystem),
    MouseWheel(Position, i32),
    Char(char),
    KeyDown(Key, JobSystem, &'i mut bool),
    KeyUp(Key),
    Hotkey(Hotkey, &'i mut bool),
    GetHotkeys(&'i mut HashMap<Hotkey, HotkeyCallback>, bool),
    Show,
    Hide,
    Create,
    Destroy,
}

pub trait GuiControl: std::fmt::Debug + 'static {
    fn get_base_mut(&mut self) -> &mut GuiControlBase;
    fn on_message(&mut self, m: GuiMessage) -> bool;
}

pub(crate) fn avg_color(color1: u32, color2: u32) -> u32 {
    (((color1 ^ color2) & 0xFEFEFE) >> 1) + (color1 & color2)
}

#[derive(Debug, Copy, Clone)]
pub struct GuiColorTheme {
    pub background: u32,
    pub font: u32,
    pub hotkey: u32,
    pub splitter: u32,
    pub highlight: u32,
    pub pressed: u32,
    pub selected: u32,
    pub inactive: u32,
    pub edit_highlight: u32,
    pub edit_focused: u32,
}

pub static DARK_THEME: GuiColorTheme = GuiColorTheme {
    background: 0x000000,
    font: 0xCCCCCC,
    hotkey: 0xAACCAA,
    splitter: 0xAACCAA,
    highlight: 0xAAAAAA,
    pressed: 0xFFFFFF,
    selected: 0x66CC66,
    inactive: 0x444444,
    edit_highlight: 0x666666,
    edit_focused: 0x888888,
};

pub static LIGHT_THEME: GuiColorTheme = GuiColorTheme {
    background: 0xFFFFFF,
    font: 0x000000,
    hotkey: 0x664422,
    splitter: 0x664422,
    highlight: 0x666666,
    pressed: 0x222222,
    selected: 0xCC8844,
    inactive: 0xAAAAAA,
    edit_highlight: 0xCCCCCC,
    edit_focused: 0xEEEEEE,
};

pub struct GuiSystem {
    job_system: JobSystem,
    root: Option<Rc<RefCell<dyn GuiControl>>>,
    focus: Option<Weak<RefCell<dyn GuiControl>>>,
    highlight: Option<Weak<RefCell<dyn GuiControl>>>,
    pressed: Option<Weak<RefCell<dyn GuiControl>>>,
    color_theme: GuiColorTheme,
    updated: bool,
    updated_hotkeys: bool,
    hotkeys: HashMap<Hotkey, HotkeyCallback>,
    global_hotkeys: HashMap<Hotkey, HotkeyCallback>,
}

macro_rules! set_property {
    ($self: ident, $new: ident, $getter: ident, $field: ident, $handle_lose: expr) => {
        let off_old_flag = |s: &mut GuiSystem| {
            if let Some(old_ptr) = s.$getter() {
                GuiSystem::mark_to_redraw(&old_ptr);
                let mut old = old_ptr.borrow_mut();
                old.get_base_mut().$field = false;
                if $handle_lose {
                    if old.on_message(GuiMessage::FocusLose(s.job_system.clone())) {
                        s.updated = false;
                        s.updated_hotkeys = false;
                    }
                }
                return true;
            } else {
                return false;
            }
        };

        if let Some(new_ptr) = $new {
            {
                GuiSystem::mark_to_redraw(&new_ptr);
                let mut new = new_ptr.borrow_mut();
                let new_base = new.get_base_mut();
                if new_base.$field {
                    return false;
                }
                new_base.$field = true;
            }
            off_old_flag($self);
            $self.$field = Some(Rc::downgrade(&new_ptr));
        } else {
            if off_old_flag($self) {
                $self.$field = None;
            } else {
                return false;
            }
        }

        return true;
    };
}

impl GuiSystem {
    pub fn new(job_system: JobSystem) -> Self {
        Self {
            job_system,
            root: None,
            focus: None,
            highlight: None,
            pressed: None,
            color_theme: LIGHT_THEME,
            updated: false,
            updated_hotkeys: false,
            hotkeys: Default::default(),
            global_hotkeys: Default::default(),
        }
    }

    pub fn set_rect(control: &mut dyn GuiControl, rect: Rect) {
        assert!(rect.right_bottom.0 >= rect.left_top.0);
        assert!(rect.right_bottom.1 >= rect.left_top.1);
        control.get_base_mut().rect = rect;
        control.on_message(GuiMessage::RectUpdated);
    }

    pub fn mark_to_redraw(control: &Rc<RefCell<dyn GuiControl>>) {
        control.borrow_mut().get_base_mut().need_redraw = true;
    }

    pub fn get_child(
        control: &Rc<RefCell<dyn GuiControl>>,
        position: Position,
    ) -> Rc<RefCell<dyn GuiControl>> {
        let mut result = control.clone();
        control
            .borrow_mut()
            .on_message(GuiMessage::FindDestination(&mut result, position));
        result
    }

    pub fn get_size_constraints(control: &mut dyn GuiControl) -> SizeConstraints {
        let mut size_constraints = control.get_base_mut().size_constraints;
        control.on_message(GuiMessage::UpdateSizeConstraints(&mut size_constraints));
        size_constraints
    }

    fn get_focus(&self) -> Option<Rc<RefCell<dyn GuiControl>>> {
        self.focus.as_ref().and_then(Weak::upgrade).clone()
    }

    fn set_focus(
        &mut self,
        new_focus: Option<Rc<RefCell<dyn GuiControl>>>,
        handle_lose: bool,
    ) -> bool {
        set_property!(self, new_focus, get_focus, focus, handle_lose);
    }

    fn get_highlight(&self) -> Option<Rc<RefCell<dyn GuiControl>>> {
        self.highlight.as_ref().and_then(Weak::upgrade).clone()
    }

    fn set_highlight(&mut self, new_highlight: Option<Rc<RefCell<dyn GuiControl>>>) -> bool {
        set_property!(self, new_highlight, get_highlight, highlight, false);
    }

    fn get_pressed(&self) -> Option<Rc<RefCell<dyn GuiControl>>> {
        self.pressed.as_ref().and_then(Weak::upgrade).clone()
    }

    fn set_pressed(&mut self, new_pressed: Option<Rc<RefCell<dyn GuiControl>>>) -> bool {
        set_property!(self, new_pressed, get_pressed, pressed, false);
    }

    pub fn on_draw(&mut self, draw_context: &mut DrawContext) {
        if let Some(root) = &self.root {
            let mut root = root.borrow_mut();
            if !self.updated {
                Self::set_rect(
                    root.deref_mut(),
                    Rect {
                        left_top: (0, 0),
                        right_bottom: image_size_to_position(draw_context.buffer.get_size()),
                    },
                );
                self.updated = true;
                root.get_base_mut().need_redraw = true;
            }

            root.on_message(GuiMessage::Draw(
                &mut draw_context.buffer,
                &self.color_theme,
                false,
            ));
        }
    }

    pub fn set_color_theme(&mut self, color_theme: GuiColorTheme) {
        self.color_theme = color_theme;
    }

    pub fn on_resize(&mut self) {
        self.updated = false;
    }

    pub fn get_minimal_size(&self) -> Position {
        if let Some(root) = &self.root {
            let mut root = root.borrow_mut();
            let size_constraints = Self::get_size_constraints(root.deref_mut());
            (size_constraints.0.absolute, size_constraints.1.absolute)
        } else {
            (0, 0)
        }
    }

    pub fn on_mouse_down(&mut self, position: Position) -> bool {
        if let Some(root) = &self.root {
            let child = Self::get_child(&root, position);
            if child
                .borrow_mut()
                .on_message(GuiMessage::MouseDown(position))
            {
                let changed_focus = self.set_focus(Some(child.clone()), true);
                let changed_pressed = self.set_pressed(Some(child.clone()));
                GuiSystem::mark_to_redraw(&child);
                return changed_focus || changed_pressed;
            } else {
                return self.set_focus(None, true);
            }
        }
        return false;
    }

    pub fn on_mouse_move(&mut self, position: Position) -> bool {
        if let Some(root) = &self.root {
            let maybe_pressed = self.get_pressed();
            let handler = if let Some(pressed) = &maybe_pressed {
                pressed.clone()
            } else {
                Self::get_child(&root, position)
            };
            let handled = handler
                .borrow_mut()
                .on_message(GuiMessage::MouseMove(position));
            let changed_highlight = self.set_highlight(Some(handler.clone()));
            if handled {
                GuiSystem::mark_to_redraw(&handler);
            }
            return handled || changed_highlight;
        }

        return false;
    }

    pub fn on_mouse_wheel(&mut self, position: Position, delta: i32) -> bool {
        if let Some(root) = &self.root {
            let maybe_pressed = self.get_pressed();
            let handler = if let Some(pressed) = &maybe_pressed {
                pressed.clone()
            } else {
                Self::get_child(&root, position)
            };
            let handled = handler
                .borrow_mut()
                .on_message(GuiMessage::MouseWheel(position, delta));
            if handled {
                GuiSystem::mark_to_redraw(&handler);
            }
            return handled;
        }

        return false;
    }

    pub fn on_mouse_leave(&mut self) -> bool {
        return self.set_highlight(None);
    }

    pub fn on_deactivate(&mut self) -> bool {
        let changed_highlight = self.set_highlight(None);
        let changed_pressed = self.set_pressed(None);
        let changed_focus = self.set_focus(None, false);
        return changed_highlight || changed_pressed || changed_focus;
    }

    pub fn on_mouse_up(&mut self, position: Position) -> bool {
        if let Some(root) = &self.root {
            let maybe_pressed = self.get_pressed();
            let handler = if let Some(pressed) = &maybe_pressed {
                pressed.clone()
            } else {
                Self::get_child(&root, position)
            };
            if handler
                .borrow_mut()
                .on_message(GuiMessage::MouseUp(position, self.job_system.clone()))
            {
                self.updated = false;
                self.updated_hotkeys = false;
                GuiSystem::mark_to_redraw(&handler);
                return self.set_pressed(None);
            }
        }
        return false;
    }

    pub fn on_char(&mut self, c: char) -> bool {
        if let Some(focus) = self.get_focus() {
            let handled = focus.borrow_mut().on_message(GuiMessage::Char(c));
            if handled {
                GuiSystem::mark_to_redraw(&focus);
            }
            return handled;
        }

        return false;
    }

    pub fn on_key_down(&mut self, k: Key) -> bool {
        if let Some(focus) = self.get_focus() {
            let mut unfocus = false;
            let handled = focus.borrow_mut().on_message(GuiMessage::KeyDown(
                k,
                self.job_system.clone(),
                &mut unfocus,
            ));
            if unfocus {
                self.updated = false;
                self.updated_hotkeys = false;
                self.set_focus(None, false);
            }
            if handled {
                GuiSystem::mark_to_redraw(&focus);
            }
            return handled;
        }

        return false;
    }

    pub fn on_hotkey(&mut self, k: Hotkey) -> bool {
        if !self.updated_hotkeys {
            self.hotkeys.clear();
            if let Some(root) = &self.root {
                root.borrow_mut()
                    .on_message(GuiMessage::GetHotkeys(&mut self.hotkeys, true));
            }
            self.updated_hotkeys = true;
        }

        if let Some(focus) = self.get_focus() {
            let mut use_default_keydown = false;
            if focus
                .borrow_mut()
                .on_message(GuiMessage::Hotkey(k, &mut use_default_keydown))
            {
                GuiSystem::mark_to_redraw(&focus);
                return true;
            }

            if use_default_keydown {
                return false;
            }
        }

        if let Some(HotkeyCallback(callback)) =
            self.global_hotkeys.get(&k).or_else(|| self.hotkeys.get(&k))
        {
            self.job_system.add_callback(callback.clone());
            self.updated = false;
            self.updated_hotkeys = false;
            return true;
        }

        return false;
    }

    pub fn on_key_up(&mut self, k: Key) -> bool {
        if let Some(focus) = self.get_focus() {
            let handled = focus.borrow_mut().on_message(GuiMessage::KeyUp(k));
            if handled {
                GuiSystem::mark_to_redraw(&focus);
            }
            return handled;
        }

        return false;
    }

    pub fn create_rc_by_control<Control: GuiControl>(
        control: Control,
    ) -> (Rc<RefCell<dyn GuiControl>>, Rc<RefCell<Control>>) {
        let typed = Rc::new(RefCell::new(control));
        let untyped: Rc<RefCell<dyn GuiControl>> = typed.clone();
        typed.borrow_mut().get_base_mut().self_ref = Some(Rc::downgrade(&untyped));
        (untyped, typed)
    }

    pub fn set_root<Control: GuiControl>(&mut self, control: Control) -> Rc<RefCell<Control>> {
        if let Some(root) = &self.root {
            let mut root = root.borrow_mut();
            if root.get_base_mut().visible {
                root.on_message(GuiMessage::Hide);
            }
            root.on_message(GuiMessage::Destroy);
        }

        let (untyped, typed) = Self::create_rc_by_control(control);
        {
            let mut untyped = untyped.borrow_mut();
            if untyped.get_base_mut().visible {
                untyped.on_message(GuiMessage::Show);
            }
            untyped.on_message(GuiMessage::Create);
        }
        self.root = Some(untyped);
        typed
    }

    pub fn default_size(text: &str, hotkey: Option<Hotkey>, font: &Font) -> SizeConstraints {
        let text_size = font.get_size(text);
        if let Some(hotkey) = hotkey {
            let hotkey_size = font.get_size(&format!("{:?}", hotkey));
            SizeConstraints(
                SizeConstraint::fixed((text_size.0 + hotkey_size.0 + text_size.1 * 3 / 2) as i32),
                SizeConstraint::fixed(text_size.1 as i32 + 2),
            )
        } else {
            SizeConstraints(
                SizeConstraint::fixed((text_size.0 + text_size.1) as i32),
                SizeConstraint::fixed(text_size.1 as i32 + 2),
            )
        }
    }
}
