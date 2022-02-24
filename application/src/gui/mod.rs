pub mod gui_components;

use crate::draw_context::*;
use crate::image::*;
use crate::job_system::*;
use crate::keys::*;

use std::cell::RefCell;
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

#[derive(Debug)]
pub struct GuiControlBase {
    pub(crate) size_constraints: SizeConstraints,
    pub(crate) current_size_constraints: SizeConstraints,
    pub(crate) self_ref: Option<Weak<RefCell<dyn GuiControl>>>,
    pub visible: bool,
    pub(crate) focus: bool,
    pub(crate) highlight: bool,
    pub(crate) pressed: bool,
    pub(crate) rect: Rect,
}

impl GuiControlBase {
    pub fn new(size_constraints: SizeConstraints) -> Self {
        Self {
            size_constraints,
            current_size_constraints: size_constraints,
            self_ref: None,
            visible: true,
            focus: false,
            highlight: false,
            pressed: false,
            rect: Rect::default(),
        }
    }
}

pub enum GuiMessage<'i, 'j> {
    Draw(&'i mut ImageViewMut<'j, u32>, &'i GuiColorTheme),
    UpdateSizeConstraints(&'i mut SizeConstraints),
    FindDestination(&'i mut Rc<RefCell<dyn GuiControl>>, Position),
    RectUpdated,
    MouseDown(Position),
    MouseMove(Position),
    MouseUp(Position, JobSystem),
    MouseWheel(Position, i32),
    Char(char),
    KeyDown(Key, JobSystem, &'i mut bool),
    KeyUp(Key),
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
    pub font: u32,
    pub splitter: u32,
    pub highlight: u32,
    pub pressed: u32,
    pub selected: u32,
    pub inactive: u32,
    pub edit_highlight: u32,
    pub edit_focused: u32,
}

pub static DARK_THEME: GuiColorTheme = GuiColorTheme {
    font: 0xCCCCCC,
    splitter: 0xAACCAA,
    highlight: 0x999999,
    pressed: 0xFFFFFF,
    selected: 0x66CC66,
    inactive: 0x444444,
    edit_highlight: 0x666666,
    edit_focused: 0x888888,
};

pub static LIGHT_THEME: GuiColorTheme = GuiColorTheme {
    font: 0x000000,
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
}

macro_rules! set_property {
    ($self: ident, $new: ident, $getter: ident, $field: ident) => {
        let off_old_flag = || {
            if let Some(old) = $self.$getter() {
                let mut old = old.borrow_mut();
                let old_base = old.get_base_mut();
                old_base.$field = false;
                return true;
            } else {
                return false;
            }
        };

        if let Some(new_ptr) = $new {
            {
                let mut new = new_ptr.borrow_mut();
                let new_base = new.get_base_mut();
                if new_base.$field {
                    return false;
                }

                new_base.$field = true;
            }
            off_old_flag();
            $self.$field = Some(Rc::downgrade(&new_ptr));
        } else {
            if off_old_flag() {
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
        }
    }

    pub fn set_rect(control: &mut dyn GuiControl, rect: Rect) {
        assert!(rect.right_bottom.0 >= rect.left_top.0);
        assert!(rect.right_bottom.1 >= rect.left_top.1);
        control.get_base_mut().rect = rect;
        control.on_message(GuiMessage::RectUpdated);
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

    fn set_focus(&mut self, new_focus: Option<Rc<RefCell<dyn GuiControl>>>) -> bool {
        set_property!(self, new_focus, get_focus, focus);
    }

    fn get_highlight(&self) -> Option<Rc<RefCell<dyn GuiControl>>> {
        self.highlight.as_ref().and_then(Weak::upgrade).clone()
    }

    fn set_highlight(&mut self, new_highlight: Option<Rc<RefCell<dyn GuiControl>>>) -> bool {
        set_property!(self, new_highlight, get_highlight, highlight);
    }

    fn get_pressed(&self) -> Option<Rc<RefCell<dyn GuiControl>>> {
        self.pressed.as_ref().and_then(Weak::upgrade).clone()
    }

    fn set_pressed(&mut self, new_pressed: Option<Rc<RefCell<dyn GuiControl>>>) -> bool {
        set_property!(self, new_pressed, get_pressed, pressed);
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
            }
            root.on_message(GuiMessage::Draw(
                &mut draw_context.buffer,
                &self.color_theme,
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
                let changed_focus = self.set_focus(Some(child.clone()));
                let changed_pressed = self.set_pressed(Some(child));
                return changed_focus || changed_pressed;
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
            let changed_highlight = self.set_highlight(Some(handler));
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
            return handler
                .borrow_mut()
                .on_message(GuiMessage::MouseWheel(position, delta));
        }

        return false;
    }

    pub fn on_mouse_leave(&mut self) -> bool {
        return self.set_highlight(None);
    }

    pub fn on_deactivate(&mut self) -> bool {
        let changed_highlight = self.set_highlight(None);
        let changed_pressed = self.set_pressed(None);
        let changed_focus = self.set_focus(None);
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
                return self.set_pressed(None);
            }
        }
        return false;
    }

    pub fn on_char(&mut self, c: char) -> bool {
        if let Some(focus) = self.get_focus() {
            return focus.borrow_mut().on_message(GuiMessage::Char(c));
        }

        return false;
    }

    pub fn on_key_down(&mut self, k: Key) -> bool {
        if let Some(focus) = self.get_focus() {
            let mut unfocus = false;
            let result = focus.borrow_mut().on_message(GuiMessage::KeyDown(
                k,
                self.job_system.clone(),
                &mut unfocus,
            ));
            if unfocus {
                self.updated = false;
                self.set_focus(None);
            }
            return result;
        }

        return false;
    }

    pub fn on_key_up(&mut self, k: Key) -> bool {
        if let Some(focus) = self.get_focus() {
            return focus.borrow_mut().on_message(GuiMessage::KeyUp(k));
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
        let (untyped, typed) = Self::create_rc_by_control(control);
        self.root = Some(untyped);
        typed
    }
}
