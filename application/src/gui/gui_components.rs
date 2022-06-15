use std::cell::RefCell;
use std::cmp::{max, min};
use std::ops::DerefMut;
use std::rc::Rc;

use crate::callback;
use crate::callback_body;
use crate::clipboard::Clipboard;
use crate::font::*;
use crate::gui::*;
use crate::image::*;
use crate::keys::*;

#[derive(Debug, Copy, Clone)]
pub enum ContainerLayout {
    Vertical,
    Horizontal,
}

#[derive(Debug)]
pub struct Container {
    base: GuiControlBase,
    layout: ContainerLayout,
    children: Vec<Rc<RefCell<dyn GuiControl>>>,
    empty_space_state: EmptySpaceState,
    support_compression: bool,
}

impl Container {
    pub fn new(size_constraints: SizeConstraints, layout: ContainerLayout) -> Self {
        Self {
            base: GuiControlBase::new(size_constraints),
            layout,
            children: Default::default(),
            empty_space_state: EmptySpaceState::Empty,
            support_compression: false,
        }
    }

    pub fn compressed(mut self) -> Self {
        self.support_compression = true;
        self
    }

    pub fn empty_space_state(mut self, empty_space_state: EmptySpaceState) -> Self {
        self.empty_space_state = empty_space_state;
        self
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn insert_child<Control: GuiControl>(
        &mut self,
        index: usize,
        control: Control,
    ) -> Rc<RefCell<Control>> {
        let (untyped, typed) = GuiSystem::create_rc_by_control(control);
        untyped.borrow_mut().on_message(GuiMessage::Create);
        if self.base.visible {
            untyped.borrow_mut().on_message(GuiMessage::Show);
        }
        self.children.insert(index, untyped);
        typed
    }

    pub fn add_child<Control: GuiControl>(&mut self, control: Control) -> Rc<RefCell<Control>> {
        let index = self.child_count();
        self.insert_child(index, control)
    }

    pub fn delete_child(&mut self, index: usize) {
        self.children.remove(index);
    }
}

macro_rules! set_layout {
    ($self: expr, $index0: tt, $index1: tt) => {
        let mut sum_relative = 0;
        let mut sum_absolute = 0;
        for child in &$self.children {
            let child_size_constraints = child.borrow_mut().get_base_mut().current_size_constraints;
            sum_relative += child_size_constraints.$index1.relative;
            sum_absolute += child_size_constraints.$index1.absolute;
        }
        let rect = $self.base.rect;
        let size = rect.right_bottom.$index1 - rect.left_top.$index1;
        let perp_size = rect.right_bottom.$index0 - rect.left_top.$index0;
        let sum_relative = max(max(sum_relative, 100), 1);
        let relative_remainder = if size > sum_absolute {
            size - sum_absolute
        } else {
            0
        };
        let sum_absolute = max(max(sum_absolute, size), 1);

        let mut current_shift = 0;
        let mut sum_child_absolute = 0;
        let mut sum_child_relative = 0;

        for child in &$self.children {
            let mut child = child.borrow_mut();
            let child_size_constraints = child.get_base_mut().current_size_constraints;
            sum_child_absolute += child_size_constraints.$index1.absolute;
            sum_child_relative += child_size_constraints.$index1.relative;

            let next_shift = sum_child_absolute * size / sum_absolute
                + sum_child_relative * relative_remainder / sum_relative;
            let mut child_rect = Rect::default();
            child_rect.left_top.$index0 = rect.left_top.$index0;
            child_rect.left_top.$index1 = rect.left_top.$index1 + current_shift;
            child_rect.right_bottom.$index0 = min(
                rect.left_top.$index0
                    + child_size_constraints.$index0.absolute
                    + (perp_size - child_size_constraints.$index0.absolute)
                        * child_size_constraints.$index0.relative
                        / 100,
                rect.right_bottom.$index0,
            );
            child_rect.right_bottom.$index1 = rect.left_top.$index1 + next_shift;
            GuiSystem::set_rect(child.deref_mut(), child_rect);
            current_shift = next_shift;
        }
    };
}

macro_rules! fill_empty_space {
    ($self: expr, $buf: expr, $theme: expr, $index0: tt, $index1: tt) => {
        let mut max_pos = $self.base.rect.left_top.$index1;
        for child in &$self.children {
            let mut child = child.borrow_mut();
            let rect = child.get_base_mut().rect;
            max_pos = max(max_pos, rect.right_bottom.$index1);

            let mut fill_rect = rect;
            fill_rect.left_top.$index0 = rect.right_bottom.$index0;
            fill_rect.right_bottom.$index0 = $self.base.rect.right_bottom.$index0;

            let mut buf_for_fill = $buf.window_mut(
                position_to_image_size($self.base.rect.relative(fill_rect.left_top)),
                position_to_image_size($self.base.rect.relative(fill_rect.right_bottom)),
            );
            GuiSystem::erase_background(&mut buf_for_fill, $self.empty_space_state, $theme);
        }

        let mut fill_rect = $self.base.rect;
        fill_rect.left_top.$index1 = max_pos;
        let mut buf_for_fill = $buf.window_mut(
            position_to_image_size($self.base.rect.relative(fill_rect.left_top)),
            position_to_image_size($self.base.rect.relative(fill_rect.right_bottom)),
        );
        GuiSystem::erase_background(&mut buf_for_fill, $self.empty_space_state, $theme);
    };
}

impl GuiControl for Container {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        match m {
            GuiMessage::FindDestination(dest, position) => {
                if let Some(child) = self
                    .children
                    .iter()
                    .find(|child| child.borrow_mut().get_base_mut().rect.contains(position))
                {
                    *dest = GuiSystem::get_child(child, position);
                }

                return true;
            }
            GuiMessage::UpdateSizeConstraints => {
                for child in &self.children {
                    child
                        .borrow_mut()
                        .on_message(GuiMessage::UpdateSizeConstraints);
                }

                let child_real_constraints: Vec<_> = self
                    .children
                    .iter_mut()
                    .map(|c| c.borrow_mut().get_base_mut().current_size_constraints)
                    .collect();

                let child_minimal_sizes: Vec<_> = self
                    .children
                    .iter_mut()
                    .map(|c| c.borrow_mut().get_base_mut().minimal_size)
                    .collect();

                let mut abs_constraints: Position = (0, 0);
                let minimal_size = &mut self.base.minimal_size;
                *minimal_size = (0, 0);
                match self.layout {
                    ContainerLayout::Vertical => {
                        for child_real_constraints in &child_real_constraints {
                            abs_constraints.0 =
                                max(abs_constraints.0, child_real_constraints.0.absolute);
                            abs_constraints.1 += child_real_constraints.1.absolute;
                        }
                        for child_minimal_sizes in &child_minimal_sizes {
                            minimal_size.0 = max(minimal_size.0, child_minimal_sizes.0);
                            if !self.support_compression {
                                minimal_size.1 += child_minimal_sizes.1;
                            }
                        }
                        minimal_size.0 = max(minimal_size.0, self.base.size_constraints.0.absolute);
                        if !self.support_compression {
                            minimal_size.1 =
                                max(minimal_size.1, self.base.size_constraints.1.absolute);
                        }
                    }
                    ContainerLayout::Horizontal => {
                        for child_real_constraints in &child_real_constraints {
                            abs_constraints.0 += child_real_constraints.0.absolute;
                            abs_constraints.1 =
                                max(abs_constraints.1, child_real_constraints.1.absolute);
                        }
                        for child_minimal_sizes in &child_minimal_sizes {
                            if !self.support_compression {
                                minimal_size.0 += child_minimal_sizes.0;
                            }
                            minimal_size.1 = max(minimal_size.1, child_minimal_sizes.1);
                        }
                        if !self.support_compression {
                            minimal_size.0 =
                                max(minimal_size.0, self.base.size_constraints.0.absolute);
                        }
                        minimal_size.1 = max(minimal_size.1, self.base.size_constraints.1.absolute);
                    }
                }

                self.base.current_size_constraints.0.absolute =
                    max(self.base.size_constraints.0.absolute, abs_constraints.0);
                self.base.current_size_constraints.1.absolute =
                    max(self.base.size_constraints.1.absolute, abs_constraints.1);

                return true;
            }
            GuiMessage::RectUpdated => {
                match self.layout {
                    ContainerLayout::Horizontal => {
                        set_layout!(self, 1, 0);
                    }
                    ContainerLayout::Vertical => {
                        set_layout!(self, 0, 1);
                    }
                }
                return true;
            }
            GuiMessage::Draw(buf, theme, force) => {
                if self.base.visible {
                    let need_force = self.base.can_draw(force);
                    for child in &self.children {
                        let mut child = child.borrow_mut();
                        let rect = child.get_base_mut().rect;
                        let mut buf_for_child = buf.window_mut(
                            position_to_image_size(self.base.rect.relative(rect.left_top)),
                            position_to_image_size(self.base.rect.relative(rect.right_bottom)),
                        );
                        child.on_message(GuiMessage::Draw(&mut buf_for_child, theme, need_force));
                    }
                    // Clear unused space
                    match self.layout {
                        ContainerLayout::Horizontal => {
                            fill_empty_space!(self, buf, theme, 1, 0);
                        }
                        ContainerLayout::Vertical => {
                            fill_empty_space!(self, buf, theme, 0, 1);
                        }
                    }
                }
                return true;
            }
            GuiMessage::GetHotkeys(hotkey_map, active) => {
                for child in &self.children {
                    let active =
                        active && self.base.visible && child.borrow_mut().get_base_mut().visible;
                    child
                        .borrow_mut()
                        .on_message(GuiMessage::GetHotkeys(hotkey_map, active));
                }
                return false;
            }
            GuiMessage::Create => {
                for child in &self.children {
                    child.borrow_mut().on_message(GuiMessage::Create);
                }
                return true;
            }
            GuiMessage::Destroy => {
                for child in &self.children {
                    child.borrow_mut().on_message(GuiMessage::Destroy);
                }
                return true;
            }
            GuiMessage::Show => {
                for child in &self.children {
                    let mut child = child.borrow_mut();
                    if child.get_base_mut().visible {
                        child.on_message(GuiMessage::Show);
                    }
                }
                return true;
            }
            GuiMessage::Hide => {
                for child in &self.children {
                    let mut child = child.borrow_mut();
                    if child.get_base_mut().visible {
                        child.on_message(GuiMessage::Hide);
                    }
                }
                return true;
            }
            _ => return false,
        }
    }
}

#[derive(Debug)]
pub struct EmptySpace {
    base: GuiControlBase,
    state: EmptySpaceState,
}

impl EmptySpace {
    pub fn new_empty(size_constraints: SizeConstraints) -> Self {
        Self {
            base: GuiControlBase::new(size_constraints),
            state: EmptySpaceState::Empty,
        }
    }

    pub fn new_inactive(size_constraints: SizeConstraints) -> Self {
        Self {
            base: GuiControlBase::new(size_constraints),
            state: EmptySpaceState::Inactive,
        }
    }

    pub fn new_splitter(size_constraints: SizeConstraints) -> Self {
        Self {
            base: GuiControlBase::new(size_constraints),
            state: EmptySpaceState::Splitter,
        }
    }
}

impl GuiControl for EmptySpace {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        match m {
            GuiMessage::Draw(buf, theme, force) => {
                if self.base.can_draw(force) {
                    GuiSystem::erase_background(buf, self.state, theme);
                }

                return true;
            }
            _ => return false,
        }
    }
}

#[derive(Clone)]
pub struct ButtonCallback(Rc<dyn Fn() + 'static>);

impl std::fmt::Debug for ButtonCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("ButtonCallback")
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum ButtonCheckState {
    None,
    CheckBox(bool),
    RadioButton(bool),
    TabButton(bool),
}

#[derive(Debug)]
pub struct Button {
    base: GuiControlBase,
    holded_when_pushed: bool,
    font: Font,
    text: String,
    hotkey: Option<Hotkey>,
    hotkey_is_global: bool,
    callback: Option<ButtonCallback>,
    check_state: Rc<RefCell<ButtonCheckState>>,
}

impl Button {
    pub fn new_with_hotkey(
        size_constraints: SizeConstraints,
        text: String,
        font: Font,
        hotkey: Option<Hotkey>,
    ) -> Self {
        Self {
            base: GuiControlBase::new(size_constraints),
            holded_when_pushed: false,
            font: font
                .layout_vertical(TextLayoutVertical::MIDDLE)
                .layout_horizontal(TextLayoutHorizontal::LEFT),
            text,
            hotkey,
            hotkey_is_global: false,
            callback: None,
            check_state: Rc::new(RefCell::new(ButtonCheckState::None)),
        }
    }

    pub fn new(size_constraints: SizeConstraints, text: String, font: Font) -> Self {
        Self::new_with_hotkey(size_constraints, text, font, None)
    }

    pub fn set_callback(&mut self, callback: impl Fn() + 'static) {
        self.callback = Some(ButtonCallback(Rc::new(callback)));
    }

    pub fn global(mut self) -> Self {
        self.hotkey_is_global = true;
        self
    }

    pub fn callback(mut self, callback: impl Fn() + 'static) -> Self {
        self.set_callback(callback);
        self
    }

    pub fn set_checkbox_callback(&mut self, callback: impl Fn(bool) + 'static) {
        let check_state = self.check_state.clone();
        self.callback = Some(ButtonCallback(Rc::new(callback!([check_state]() {
            callback(*check_state.borrow() == ButtonCheckState::CheckBox(true));
        }))));
    }

    pub fn checkbox_callback(mut self, callback: impl Fn(bool) + 'static) -> Self {
        self.set_checkbox_callback(callback);
        self
    }

    pub fn check_box(self, state: bool) -> Self {
        *self.check_state.borrow_mut() = ButtonCheckState::CheckBox(state);
        self
    }

    pub fn radio_button(self) -> Self {
        *self.check_state.borrow_mut() = ButtonCheckState::RadioButton(false);
        self
    }

    pub fn tab_button(self) -> Self {
        *self.check_state.borrow_mut() = ButtonCheckState::TabButton(false);
        self
    }

    pub fn hotkey(mut self, hotkey: Hotkey, global: bool) -> Self {
        self.hotkey = Some(hotkey);
        self.hotkey_is_global = global;
        self
    }

    fn has_checks(&self) -> bool {
        match *self.check_state.borrow() {
            ButtonCheckState::None => false,
            ButtonCheckState::CheckBox(_) => true,
            ButtonCheckState::RadioButton(_) => true,
            ButtonCheckState::TabButton(_) => false,
        }
    }

    fn checked(&self) -> bool {
        match *self.check_state.borrow() {
            ButtonCheckState::None => false,
            ButtonCheckState::CheckBox(c) => c,
            ButtonCheckState::RadioButton(c) => c,
            ButtonCheckState::TabButton(c) => c,
        }
    }

    pub fn set_checked(&mut self, checked: bool) {
        match self.check_state.borrow_mut().deref_mut() {
            ButtonCheckState::None => {}
            ButtonCheckState::CheckBox(c) => *c = checked,
            ButtonCheckState::RadioButton(c) => *c = checked,
            ButtonCheckState::TabButton(c) => *c = checked,
        }
    }

    fn default_check_symbol_of_state(state: ButtonCheckState) -> &'static str {
        match state {
            ButtonCheckState::None => "",
            ButtonCheckState::CheckBox(_) => "V",
            ButtonCheckState::RadioButton(_) => "●",
            ButtonCheckState::TabButton(_) => "",
        }
    }

    fn default_check_symbol(&self) -> &'static str {
        Self::default_check_symbol_of_state(*self.check_state.borrow())
    }

    fn check_symbol(&self) -> &'static str {
        match *self.check_state.borrow() {
            ButtonCheckState::None => "",
            ButtonCheckState::CheckBox(c) => {
                if c {
                    "V"
                } else {
                    "□"
                }
            }
            ButtonCheckState::RadioButton(c) => {
                if c {
                    "●"
                } else {
                    "○"
                }
            }
            ButtonCheckState::TabButton(_) => "",
        }
    }

    pub fn get_background_color(&self, theme: &GuiColorTheme) -> u32 {
        let color = match *self.check_state.borrow() {
            ButtonCheckState::TabButton(checked) => {
                if checked {
                    theme.background
                } else {
                    theme.inactive
                }
            }
            _ => {
                if self.checked() {
                    avg_color(theme.background, theme.selected)
                } else {
                    theme.background
                }
            }
        };

        if self.base.pressed {
            avg_color(color, theme.pressed)
        } else if self.base.highlight {
            avg_color(color, theme.highlight)
        } else {
            color
        }
    }

    pub fn default_size(text: &str, hotkey: Option<Hotkey>, font: &Font) -> SizeConstraints {
        GuiSystem::default_size(text, hotkey, font)
    }

    pub fn default_checkbox_size(
        text: &str,
        hotkey: Option<Hotkey>,
        font: &Font,
    ) -> SizeConstraints {
        let check_size = font.get_size(Self::default_check_symbol_of_state(
            ButtonCheckState::CheckBox(true),
        ));
        let mut result = Self::default_size(text, hotkey, font);
        result.0.absolute += check_size.0 as i32;
        result
    }

    pub fn default_radiobutton_size(
        text: &str,
        hotkey: Option<Hotkey>,
        font: &Font,
    ) -> SizeConstraints {
        let radio_size = font.get_size(Self::default_check_symbol_of_state(
            ButtonCheckState::RadioButton(true),
        ));
        let mut result = Self::default_size(text, hotkey, font);
        result.0.absolute += radio_size.0 as i32;
        result
    }
}

impl GuiControl for Button {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        match m {
            GuiMessage::Draw(buf, theme, force) => {
                if self.base.can_draw(force) {
                    let size = buf.get_size();
                    if size.0 > 0 && size.1 > 0 {
                        // Calculate layout of checkbox, caption hotkey
                        let color = self.get_background_color(theme);
                        buf.fill(|d| *d = color);

                        let default_check_symbol = self.default_check_symbol();
                        let check_size = self.font.get_size(default_check_symbol);
                        let check_width = min(check_size.0 + check_size.1 / 2, buf.get_size().0);

                        let hotkey_text = if let Some(hotkey) = self.hotkey {
                            format!("{:?}", hotkey)
                        } else {
                            "".to_string()
                        };

                        let hotkey_size = self.font.get_size(&hotkey_text);
                        let hotkey_width = min(
                            hotkey_size.0 + hotkey_size.1 / 2,
                            buf.get_size().0 - check_width,
                        );

                        // Draw caption
                        let mut caption_dst = buf.window_mut(
                            (check_width, 0),
                            (buf.get_size().0 - hotkey_width, buf.get_size().1),
                        );
                        let with_checks = self.has_checks();
                        let text_size = self.font.get_size(&self.text);
                        let text_too_long = text_size.0 > caption_dst.get_size().0;
                        let caption_text = if text_too_long {
                            let get_longest_text =
                                |input: &str, font: &Font, max_size: usize| -> String {
                                    let dots = "...";
                                    let mut good_text = String::new();
                                    let mut suffix = "";
                                    let mut size = font.get_size(dots).1 / 2;
                                    let mut tmp = [0u8; 4];
                                    for (index, c) in input.chars().enumerate() {
                                        size += font.get_size(c.encode_utf8(&mut tmp)).0;
                                        if size > max_size && index > 0 {
                                            return good_text + suffix;
                                        }
                                        good_text.push(c);
                                        if index == 0 {
                                            size += font.get_size(dots).0;
                                            if size > max_size {
                                                return good_text;
                                            }
                                            suffix = dots;
                                        }
                                    }
                                    return good_text + suffix;
                                };

                            get_longest_text(&self.text, &self.font, caption_dst.get_size().0)
                        } else {
                            self.text.clone()
                        };

                        let mut caption_position = (
                            if with_checks {
                                0
                            } else {
                                (caption_dst.get_size().0 - check_width + text_size.1 % 2) as i32
                                    / 2
                            },
                            caption_dst.get_size().1 as i32 / 2,
                        );
                        if self.base.pressed && self.holded_when_pushed {
                            caption_position.0 += 1;
                            caption_position.1 += 1;
                        }

                        let font = if with_checks {
                            self.font.clone()
                        } else {
                            self.font.layout_horizontal(TextLayoutHorizontal::MIDDLE)
                        };
                        font.color(theme.font).draw(
                            &caption_text,
                            caption_position,
                            &mut caption_dst,
                        );

                        // Draw hotkey
                        let mut hotkey_text_dst =
                            buf.window_mut((buf.get_size().0 - hotkey_width, 0), buf.get_size());
                        let hotkey_position = (0, hotkey_text_dst.get_size().1 as i32 / 2);

                        self.font
                            .layout_horizontal(TextLayoutHorizontal::LEFT)
                            .color(theme.splitter)
                            .draw(&hotkey_text, hotkey_position, &mut hotkey_text_dst);

                        // Draw checkbox or radiobox
                        let check_symbol = self.check_symbol();

                        let mut check_dst = buf.window_mut((0, 0), (check_width, buf.get_size().1));
                        let check_position = (
                            check_dst.get_size().0 as i32 / 2,
                            check_dst.get_size().1 as i32 / 2,
                        );
                        self.font
                            .color(theme.font)
                            .layout_horizontal(TextLayoutHorizontal::MIDDLE)
                            .draw(check_symbol, check_position, &mut check_dst);
                    }
                }
                return false;
            }
            GuiMessage::MouseDown(_) => {
                self.holded_when_pushed = true;
                return true;
            }
            GuiMessage::MouseMove(position) => {
                if self.base.pressed {
                    let prev_holded_when_pushed = self.holded_when_pushed;
                    self.holded_when_pushed = self.base.rect.contains(position);
                    return self.holded_when_pushed != prev_holded_when_pushed;
                } else {
                    return false;
                }
            }
            GuiMessage::MouseUp(position, job_system) => {
                if self.base.pressed && self.base.rect.contains(position) {
                    if let ButtonCheckState::CheckBox(c) =
                        &mut self.check_state.borrow_mut().deref_mut()
                    {
                        *c = !*c;
                    }
                    if let Some(ButtonCallback(callback)) = &self.callback {
                        job_system.add_callback(callback.clone());
                    }
                }
                return true;
            }
            GuiMessage::GetHotkeys(hotkey_map, active) => {
                if let Some(hotkey) = self.hotkey {
                    if let Some(ButtonCallback(callback)) = &self.callback {
                        if self.hotkey_is_global || active {
                            hotkey_map.insert(hotkey, HotkeyCallback(callback.clone()));
                        }
                        return true;
                    }
                }
                return false;
            }
            _ => return false,
        }
    }
}

#[derive(Debug)]
pub struct TextBox {
    base: GuiControlBase,
    padding: bool,
    font: Font,
    text: String,
}

impl TextBox {
    pub fn new(size_constraints: SizeConstraints, text: String, font: Font) -> Self {
        Self {
            base: GuiControlBase::new(size_constraints),
            padding: true,
            font: font
                .layout_vertical(TextLayoutVertical::MIDDLE)
                .layout_horizontal(TextLayoutHorizontal::LEFT),
            text,
        }
    }
}

impl GuiControl for TextBox {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        match m {
            GuiMessage::Draw(buf, theme, force) => {
                if self.base.can_draw(force) {
                    GuiSystem::erase_background(buf, EmptySpaceState::Empty, theme);
                    let size = buf.get_size();
                    if size.0 > 0 && size.1 > 0 {
                        let position = if self.padding {
                            self.font.get_size(&self.text).1 as i32 / 2
                        } else {
                            0
                        };
                        let caption_position = (position, buf.get_size().1 as i32 / 2);
                        self.font
                            .color(theme.font)
                            .draw(&self.text, caption_position, buf);
                    }
                }
                return false;
            }
            _ => return false,
        }
    }
}

#[derive(Clone)]
pub struct SkipCallback(Rc<dyn Fn() + 'static>);

impl std::fmt::Debug for SkipCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("SkipCallback")
    }
}

#[derive(Clone)]
pub struct EnterCallback(Rc<dyn Fn(&str) + 'static>);

impl std::fmt::Debug for EnterCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("EnterCallback")
    }
}

#[derive(Debug)]
pub struct Edit {
    base: GuiControlBase,
    text: Vec<char>,
    font: Font,
    clipboard: Clipboard,
    scroll_position: i32,
    cursor_position: i32,
    skip_callback: Option<SkipCallback>,
    enter_callback: Option<EnterCallback>,
    suka_down: bool,
}

impl Edit {
    pub fn new(size_constraints: SizeConstraints, font: Font, clipboard: Clipboard) -> Self {
        Self {
            base: GuiControlBase::new(size_constraints),
            text: Default::default(),
            font: font
                .layout_vertical(TextLayoutVertical::MIDDLE)
                .layout_horizontal(TextLayoutHorizontal::LEFT),
            clipboard,
            scroll_position: 0,
            cursor_position: 0,
            skip_callback: None,
            enter_callback: None,
            suka_down: false,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.chars().collect();
        self.adjust_cursor_position();
    }

    pub fn text(mut self, text: &str) -> Self {
        self.set_text(text);
        self
    }

    pub fn set_skip_callback(&mut self, callback: impl Fn() + 'static) {
        self.skip_callback = Some(SkipCallback(Rc::new(callback)));
    }

    pub fn skip_callback(mut self, skip_callback: impl Fn() + 'static) -> Self {
        self.set_skip_callback(skip_callback);
        self
    }

    pub fn set_enter_callback(&mut self, enter_callback: impl Fn(&str) + 'static) {
        self.enter_callback = Some(EnterCallback(Rc::new(enter_callback)));
    }

    pub fn enter_callback(mut self, enter_callback: impl Fn(&str) + 'static) -> Self {
        self.set_enter_callback(enter_callback);
        self
    }

    pub fn set_cursor_position(&mut self, cursor_position: i32) {
        self.cursor_position = cursor_position;
        self.adjust_cursor_position();
    }

    fn adjust_cursor_position(&mut self) {
        if self.cursor_position < self.scroll_position + 1 {
            self.scroll_position = max(0, self.cursor_position - 1);
            return;
        }

        let width = self.base.rect.right_bottom.0 - self.base.rect.left_top.0 - 4;
        let mut minimal_scroll = min(self.cursor_position, self.text.len() as i32);
        let mut width_before_cursor = 0;
        let mut buffer = [0u8; 4];
        while minimal_scroll > self.scroll_position {
            width_before_cursor += self
                .font
                .get_size(self.text[(minimal_scroll - 1) as usize].encode_utf8(&mut buffer))
                .0 as i32;
            if width_before_cursor > width {
                break;
            }

            minimal_scroll -= 1;
        }

        self.scroll_position = minimal_scroll;
    }

    fn char_is_valid(c: char) -> bool {
        match c {
            '\x00'..='\x1f' => false,
            _ => return true,
        }
    }

    fn paste(&mut self) -> bool {
        if let Some(text) = self.clipboard.get_string() {
            for c in text.chars() {
                if Self::char_is_valid(c) {
                    self.text.insert(self.cursor_position as usize, c);
                    self.cursor_position += 1;
                }
            }

            self.adjust_cursor_position();
            return true;
        }

        return false;
    }
}

impl GuiControl for Edit {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        match m {
            GuiMessage::Draw(buf, theme, force) => {
                if self.base.can_draw(force) {
                    let (x, y) = buf.get_size();
                    if x > 2 && y > 2 {
                        let border_color = theme.font;
                        buf.window_mut((0, 0), (x, 1)).fill(|p| *p = border_color);
                        buf.window_mut((0, y - 1), (x, y))
                            .fill(|p| *p = border_color);
                        buf.window_mut((0, 1), (1, y - 1))
                            .fill(|p| *p = border_color);
                        buf.window_mut((x - 1, 1), (x, y - 1))
                            .fill(|p| *p = border_color);
                        let mut inner = buf.window_mut((1, 1), (x - 1, y - 1));

                        let color = if self.base.focus {
                            avg_color(theme.background, theme.edit_focused)
                        } else {
                            avg_color(theme.background, theme.inactive)
                        };

                        let color = if self.base.highlight {
                            avg_color(color, theme.highlight)
                        } else {
                            color
                        };

                        inner.fill(|d| *d = color);
                        if self.base.focus {
                            let text_before_cursor: String = self.text
                                [self.scroll_position as usize..self.cursor_position as usize]
                                .iter()
                                .collect();
                            let cursor_position = self.font.get_size(&text_before_cursor).0;
                            if cursor_position + 2 <= x - 2 {
                                inner
                                    .window_mut((cursor_position, 0), (cursor_position + 2, y - 2))
                                    .fill(|p| *p = border_color);
                            }
                        }

                        let mut visible_text = String::new();
                        let mut width = 0;
                        let mut pos = self.scroll_position as usize;
                        loop {
                            if pos >= self.text.len() {
                                break;
                            }

                            let mut buffer = [0u8; 4];
                            width += self
                                .font
                                .get_size(self.text[pos].encode_utf8(&mut buffer))
                                .0;
                            visible_text.push(self.text[pos]);
                            if width > x - 2 {
                                break;
                            }
                            pos += 1;
                        }

                        self.font.color(theme.font).draw(
                            &visible_text,
                            (1, (y / 2) as i32),
                            &mut inner,
                        );
                    }
                }

                return true;
            }
            GuiMessage::MouseDown(position) => {
                let mut width_before_mouse = 0;
                let mut char_index = self.scroll_position;
                let mut buffer = [0u8; 4];
                let cursor_x = position.0 - self.base.rect.left_top.0 - 1;
                while char_index < self.text.len() as i32 {
                    let new_symbol_width = self
                        .font
                        .get_size(self.text[char_index as usize].encode_utf8(&mut buffer))
                        .0 as i32;
                    if width_before_mouse + new_symbol_width / 2 > cursor_x {
                        break;
                    }
                    char_index += 1;
                    width_before_mouse += new_symbol_width;
                }
                self.cursor_position = char_index;
                self.suka_down = true;
                return true;
            }
            GuiMessage::MouseUp(_, _) => {
                return true;
            }
            GuiMessage::Char(c) => {
                if Self::char_is_valid(c) {
                    self.text.insert(self.cursor_position as usize, c);
                    self.cursor_position += 1;
                    self.adjust_cursor_position();
                    return true;
                }
                return false;
            }
            GuiMessage::Hotkey(hk, use_default_keydown) => {
                if hk.no_modifiers() {
                    // ignore hotkey, use default handler
                    *use_default_keydown = true;
                    return false;
                }

                if hk == Hotkey::ctrl(Key::V) || hk == Hotkey::shift(Key::Insert) {
                    *use_default_keydown = true;
                    return self.paste();
                }

                return false;
            }
            GuiMessage::KeyDown(k, job_system, unfocus) => {
                match k {
                    Key::Left => {
                        if self.cursor_position > 0 {
                            self.cursor_position -= 1;
                            self.adjust_cursor_position();
                            return true;
                        }
                    }
                    Key::Right => {
                        if self.cursor_position < self.text.len() as i32 {
                            self.cursor_position += 1;
                            self.adjust_cursor_position();
                            return true;
                        }
                    }
                    Key::Delete => {
                        if self.cursor_position < self.text.len() as i32 {
                            self.text.remove(self.cursor_position as usize);
                            self.adjust_cursor_position();
                            return true;
                        }
                    }
                    Key::Backspace => {
                        if self.cursor_position > 0 {
                            self.text.remove(self.cursor_position as usize - 1);
                            self.cursor_position -= 1;
                            self.adjust_cursor_position();
                            return true;
                        }
                    }
                    Key::Insert => return self.paste(),
                    Key::Escape => {
                        if let Some(SkipCallback(skip_callback)) = &self.skip_callback {
                            job_system.add_callback(skip_callback.clone());
                        }
                        *unfocus = true;
                        return true;
                    }
                    Key::Enter => {
                        if let Some(EnterCallback(enter_callback)) = &self.enter_callback {
                            let text: String = self.text.iter().collect();
                            job_system.add_callback(Rc::new(callback!([enter_callback] () {
                                enter_callback(&text);
                            })));
                        }
                        *unfocus = true;
                        return true;
                    }
                    _ => {}
                }

                return false;
            }
            GuiMessage::FocusLose(job_system) => {
                if let Some(EnterCallback(enter_callback)) = &self.enter_callback {
                    let text: String = self.text.iter().collect();
                    job_system.add_callback(Rc::new(callback!([enter_callback] () {
                        enter_callback(&text);
                    })));
                    return true;
                }
                return false;
            }
            _ => return false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum ScrollState {
    Invalid,
    Less,
    OnButton(i32, i32),
    Greater,
}

#[derive(Debug)]
pub struct ScrollV {
    base: GuiControlBase,

    scroll_range: i32,
    content_size: i32,
    scroll_position: i32,
    scroll_state: ScrollState,
}

impl ScrollV {
    pub fn new(size_constraints: SizeConstraints) -> Self {
        Self {
            base: GuiControlBase::new(size_constraints),
            scroll_range: 100,
            content_size: 10,
            scroll_position: 45,
            scroll_state: ScrollState::Invalid,
        }
    }

    fn get_button_size_and_shift(&self) -> (i32, i32) {
        let height = self.base.rect.right_bottom.1 - self.base.rect.left_top.1;
        let button_size = max(5, self.content_size * height / self.scroll_range);
        let button_position = self.scroll_position * (height - button_size)
            / max(1, self.scroll_range - self.content_size);
        (button_size, button_position)
    }

    fn get_button_rect(&self) -> Rect {
        let (button_size, button_shift) = self.get_button_size_and_shift();
        let mut result = self.base.rect;
        result.left_top.1 += button_shift;
        result.right_bottom.1 = result.left_top.1 + button_size;
        result
    }

    fn very_small(&self) -> bool {
        self.base.rect.right_bottom.1 - self.base.rect.left_top.1 < 10
    }

    fn set_in_button(&mut self, position: Position) {
        let button_rect = self.get_button_rect();
        self.scroll_state = if self.very_small() {
            ScrollState::Invalid
        } else if position.1 < button_rect.left_top.1 {
            ScrollState::Less
        } else if position.1 >= button_rect.right_bottom.1 {
            ScrollState::Greater
        } else {
            ScrollState::OnButton(
                position.1 - button_rect.left_top.1,
                button_rect.right_bottom.1 - position.1,
            )
        };
    }
}

impl GuiControl for ScrollV {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        match m {
            GuiMessage::Draw(buf, theme, force) => {
                if self.base.can_draw(force) {
                    self.scroll_range = max(1, self.scroll_range);
                    self.content_size = max(1, min(self.content_size, self.scroll_range));
                    self.scroll_position = max(
                        0,
                        min(self.scroll_range - self.content_size, self.scroll_position),
                    );

                    if self.content_size >= self.scroll_range {
                        return true;
                    }

                    if self.very_small() {
                        buf.fill(|d| *d = avg_color(*d, theme.inactive));
                        return true;
                    }

                    let (button_size, button_shift) = self.get_button_size_and_shift();

                    macro_rules! top_scroll {
                        () => {
                            buf.window_mut((0, 0), (buf.get_size().0, button_shift as usize))
                        };
                    }

                    macro_rules! button_scroll {
                        () => {
                            buf.window_mut(
                                (0, button_shift as usize),
                                (buf.get_size().0, (button_size + button_shift) as usize),
                            )
                        };
                    }

                    macro_rules! bottom_scroll {
                        () => {
                            buf.window_mut(
                                (0, (button_size + button_shift) as usize),
                                buf.get_size(),
                            )
                        };
                    }

                    if self.base.pressed {
                        top_scroll!().fill(|d| *d = avg_color(*d, theme.highlight));
                        bottom_scroll!().fill(|d| *d = avg_color(*d, theme.highlight));
                        button_scroll!().fill(|d| *d = avg_color(*d, theme.pressed));
                    } else if self.base.highlight {
                        top_scroll!().fill(|d| *d = avg_color(*d, theme.inactive));
                        bottom_scroll!().fill(|d| *d = avg_color(*d, theme.inactive));
                        button_scroll!().fill(|d| *d = avg_color(*d, theme.highlight));
                    } else {
                        button_scroll!().fill(|d| *d = avg_color(*d, theme.inactive));
                    }
                }
                return true;
            }
            GuiMessage::MouseWheel(_, delta) => {
                let old_scroll_position = self.scroll_position;
                self.scroll_position = max(
                    0,
                    min(
                        self.scroll_range - self.content_size,
                        self.scroll_position + delta,
                    ),
                );
                return old_scroll_position != self.scroll_position;
            }
            GuiMessage::MouseDown(position) => {
                self.set_in_button(position);
                match self.scroll_state {
                    ScrollState::Less => self.scroll_position -= self.content_size,
                    ScrollState::Greater => self.scroll_position += self.content_size,
                    _ => {}
                }

                self.scroll_position = max(
                    0,
                    min(self.scroll_range - self.content_size, self.scroll_position),
                );
                return true;
            }
            GuiMessage::MouseMove(position) => {
                if self.base.pressed {
                    match self.scroll_state {
                        ScrollState::OnButton(yl, yh) => {
                            let prev_scroll_position = self.scroll_position;
                            let (button_size, button_shift) = self.get_button_size_and_shift(); // before shift
                            let free_scroll_size = self.base.rect.right_bottom.1
                                - self.base.rect.left_top.1
                                - button_size;
                            if yl + yh > 0 && free_scroll_size > 0 {
                                let old_position = self.base.rect.left_top.1
                                    + button_shift
                                    + (button_size * yl + (yl + yh) / 2) / (yl + yh);
                                let screen_delta = position.1 - old_position;
                                let delta = (screen_delta
                                    * (self.scroll_range - self.content_size)
                                    + (if screen_delta >= 0 { 1 } else { -1 }) * free_scroll_size
                                        / 2)
                                    / max(1, free_scroll_size);
                                self.scroll_position += delta;
                                self.scroll_position = max(
                                    0,
                                    min(
                                        self.scroll_range - self.content_size,
                                        self.scroll_position,
                                    ),
                                );
                            }
                            return prev_scroll_position != self.scroll_position;
                        }
                        _ => {
                            return false;
                        }
                    }
                } else {
                    return false;
                }
            }
            GuiMessage::MouseUp(_, _) => {
                return true;
            }
            _ => return false,
        }
    }
}

#[derive(Debug)]
pub struct ListBox {
    base: GuiControlBase,
    scroll_width: i32,
    scroll: ScrollV,
    font: Font,
    pub lines: Vec<String>,
}

impl ListBox {
    pub fn new(size_constraints: SizeConstraints, scroll_width: i32, font: Font) -> Self {
        Self {
            base: GuiControlBase::new(size_constraints),
            scroll_width,
            scroll: ScrollV::new(size_constraints),
            font: font
                .layout_vertical(TextLayoutVertical::TOP)
                .layout_horizontal(TextLayoutHorizontal::LEFT),
            lines: Default::default(),
        }
    }

    fn get_item_height(&self) -> i32 {
        self.font.get_size("M").1 as i32 * 5 / 4
    }
}

impl GuiControl for ListBox {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        match m {
            GuiMessage::RectUpdated => {
                let mut scroll_rect = &mut self.scroll.base.rect;
                *scroll_rect = self.base.rect;
                scroll_rect.left_top.0 = max(
                    scroll_rect.left_top.0,
                    scroll_rect.right_bottom.0 - self.scroll_width,
                );
                return true;
            }
            GuiMessage::Draw(buf, theme, force) => {
                if self.base.can_draw(force) {
                    self.scroll.base.highlight = self.base.highlight;
                    self.scroll.base.focus = self.base.focus;
                    self.scroll.base.pressed = self.base.pressed;
                    let total_height = self.base.rect.right_bottom.1 - self.base.rect.left_top.1;
                    let item_count = (total_height + self.get_item_height() / 2)
                        / max(self.get_item_height(), 1);

                    self.scroll.content_size = item_count - 1;
                    self.scroll.scroll_range = self.lines.len() as i32;
                    let scroll_rect = self.scroll.base.rect;
                    let mut buf_for_child = buf.window_mut(
                        position_to_image_size(self.base.rect.relative(scroll_rect.left_top)),
                        position_to_image_size(self.base.rect.relative(scroll_rect.right_bottom)),
                    );
                    let scroll_result =
                        self.scroll
                            .on_message(GuiMessage::Draw(&mut buf_for_child, theme, force));

                    let first_line = min(
                        max(0, self.scroll.scroll_position) as usize,
                        self.lines.len(),
                    );
                    let last_line = min(first_line + (item_count + 1) as usize, self.lines.len());
                    let mut position = (0, 0);
                    for line in self.lines[first_line..last_line].iter() {
                        self.font.color(theme.font).draw(&line, position, buf);
                        position.1 += self.get_item_height();
                    }

                    return scroll_result;
                }
                return true;
            }
            _ => return self.scroll.on_message(m),
        }
    }
}

#[derive(Clone)]
pub struct RadioGroupCallback(Rc<dyn Fn(usize) + 'static>);

impl std::fmt::Debug for RadioGroupCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("RadioGroupCallback")
    }
}

#[derive(Debug)]
struct RadioGroupInternal {
    buttons: Vec<Weak<RefCell<Button>>>,
    close_buttons: Vec<Weak<RefCell<Button>>>,
    selected_id: usize,
    callback: Option<RadioGroupCallback>,
    selection_history: Vec<usize>,
    ids: Vec<usize>,
}

#[derive(Debug)]
pub struct RadioGroup {
    container: Container,
    internal: Rc<RefCell<RadioGroupInternal>>,
    tab: bool,
    with_closes: bool,
    last_id: usize,
}

impl RadioGroup {
    fn new_as(
        size_constraints: SizeConstraints,
        layout: ContainerLayout,
        caption: Option<TextBox>,
        tab: bool,
        with_closes: bool,
    ) -> Self {
        let mut container = Container::new(size_constraints, layout);
        if tab {
            container = container.empty_space_state(EmptySpaceState::Inactive);
        }

        caption.map(|caption| container.add_child(caption));

        Self {
            container,
            internal: Rc::new(RefCell::new(RadioGroupInternal {
                buttons: Vec::new(),
                close_buttons: Vec::new(),
                selected_id: 0,
                callback: None,
                selection_history: Vec::new(),
                ids: Vec::new(),
            })),
            tab,
            with_closes,
            last_id: 0,
        }
    }

    pub fn new(
        size_constraints: SizeConstraints,
        layout: ContainerLayout,
        caption: Option<TextBox>,
    ) -> Self {
        Self::new_as(size_constraints, layout, caption, false, false)
    }

    pub fn compressed(mut self) -> Self {
        self.container = self.container.compressed();
        self
    }

    pub fn get_id(&self) -> usize {
        self.internal.borrow().selected_id
    }

    pub fn set_callback(&mut self, callback: impl Fn(usize) + 'static) {
        self.internal.borrow_mut().callback = Some(RadioGroupCallback(Rc::new(callback)));
    }

    pub fn callback(mut self, callback: impl Fn(usize) + 'static) -> Self {
        self.set_callback(callback);
        self
    }

    fn find_index_by(ids: &[usize], id: usize) -> Option<usize> {
        ids.iter().position(|&element| element == id)
    }

    fn find_index(&self, id: usize) -> Option<usize> {
        Self::find_index_by(&self.internal.borrow().ids, id)
    }

    fn get_last_id(&self) -> usize {
        self.last_id
    }

    fn change_id_by(radio_group_internal: &mut RadioGroupInternal, new_id: usize) {
        let buttons = &mut radio_group_internal.buttons;
        let close_buttons = &mut radio_group_internal.close_buttons;
        let set_state = |b: &Weak<RefCell<Button>>, state: bool| {
            b.upgrade().map(|b| b.borrow_mut().set_checked(state));
        };

        if let Some(selection_position) = radio_group_internal
            .selection_history
            .iter()
            .position(|&id| id == new_id)
        {
            radio_group_internal
                .selection_history
                .remove(selection_position);
        }

        if let Some(old_index) =
            Self::find_index_by(&radio_group_internal.ids, radio_group_internal.selected_id)
        {
            buttons.get(old_index).map(|b| set_state(b, false));
            close_buttons.get(old_index).map(|b| set_state(b, false));
        }

        if let Some(new_index) = Self::find_index_by(&radio_group_internal.ids, new_id) {
            radio_group_internal.selected_id = new_id;
            radio_group_internal.selection_history.push(new_id);
            buttons.get(new_index).map(|b| set_state(b, true));
            close_buttons.get(new_index).map(|b| set_state(b, true));
            if let Some(RadioGroupCallback(callback)) = &radio_group_internal.callback {
                callback(new_id);
            }
        }
    }

    pub fn set_id(&self, new_id: usize) {
        Self::change_id_by(self.internal.borrow_mut().deref_mut(), new_id);
    }

    fn add_button_with_close_callback(
        &mut self,
        button: Button,
        close_callback: impl Fn() + 'static,
    ) {
        let last_id = self.last_id;
        let mut button = if self.tab {
            button.tab_button()
        } else {
            button.radio_button()
        };

        let button_font = button.font.clone();
        let close_text = "x";
        let close_text_size = button_font.get_size(close_text);
        let close_button_size = (close_text_size.0 as i32 * 2, close_text_size.1 as i32 + 2);
        let mut holder_constraints = button.base.size_constraints;
        if self.with_closes {
            holder_constraints.0.absolute += close_button_size.0;
        }

        let new_button_holder = self.container.add_child(Container::new(
            holder_constraints,
            ContainerLayout::Horizontal,
        ));

        button.base.set_size_constaints(SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::flexible(0),
        ));

        let internal = self.internal.clone();
        let new_button =
            new_button_holder
                .borrow_mut()
                .add_child(button.callback(callback!( [internal] () {
                    Self::change_id_by(
                        internal.borrow_mut().deref_mut(),
                        last_id,
                    );
                })));

        let first;
        {
            let mut internal = self.internal.borrow_mut();
            internal.buttons.push(Rc::downgrade(&new_button));
            if self.with_closes {
                let close_button_holder = new_button_holder.borrow_mut().add_child(
                    Container::new(
                        SizeConstraints(
                            SizeConstraint::fixed(close_button_size.0 as i32),
                            SizeConstraint::fixed(close_button_size.1 as i32),
                        ),
                        ContainerLayout::Vertical,
                    )
                    .empty_space_state(EmptySpaceState::Inactive),
                );

                let button = close_button_holder.borrow_mut().add_child(
                    Button::new(
                        SizeConstraints(
                            SizeConstraint::flexible(0),
                            SizeConstraint::fixed(holder_constraints.1.absolute * 75 / 100),
                        ),
                        close_text.to_string(),
                        button_font,
                    )
                    .tab_button(),
                );

                button.borrow_mut().set_callback(close_callback);
                internal.close_buttons.push(Rc::downgrade(&button));
            }
            first = internal.ids.is_empty();
            internal.ids.push(last_id);
        }
        if first {
            self.set_id(self.last_id);
        }
        self.last_id += 1;
    }

    pub fn add_button(&mut self, button: Button) {
        self.add_button_with_close_callback(button, || {});
    }

    pub fn delete_button(&mut self, id: usize) {
        if let Some(index) = self.find_index(id) {
            self.container.delete_child(index);
            let prev_id;
            {
                let mut internal = self.internal.borrow_mut();
                if index < internal.buttons.len() {
                    internal.buttons.remove(index);
                }
                if index < internal.close_buttons.len() {
                    internal.close_buttons.remove(index);
                }
                if index < internal.ids.len() {
                    internal.ids.remove(index);
                }
                if Some(id) == internal.selection_history.last().map(|id| *id) {
                    internal.selection_history.pop();
                    prev_id = internal
                        .selection_history
                        .last()
                        .or_else(|| internal.ids.get(0))
                        .cloned();
                } else {
                    prev_id = None;
                }
            }
            if let Some(prev_id) = prev_id {
                self.set_id(prev_id);
            }
        }
    }
}

impl GuiControl for RadioGroup {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.container.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        self.container.on_message(m)
    }
}

#[derive(Debug)]
pub struct TabControl {
    base: GuiControlBase,
    height: i32,
    header: RadioGroup,
    children: Vec<Rc<RefCell<dyn GuiControl>>>,
    font: Font,
}

impl TabControl {
    pub fn new(height: i32, font: Font, with_closes: bool) -> Self {
        let header_constrains =
            SizeConstraints(SizeConstraint::flexible(0), SizeConstraint::fixed(height));
        let full_constrains =
            SizeConstraints(SizeConstraint::flexible(0), SizeConstraint::fixed(height));
        Self {
            base: GuiControlBase::new(full_constrains),
            height,
            header: RadioGroup::new_as(
                header_constrains,
                ContainerLayout::Horizontal,
                None,
                true,
                with_closes,
            ),
            children: vec![],
            font,
        }
    }

    pub fn compressed(mut self) -> Self {
        self.header = self.header.compressed();
        self
    }

    pub fn add_tab_with_id<Control: GuiControl>(
        &mut self,
        caption: String,
        width: i32,
        control: Control,
        close_callback: impl Fn(usize) + 'static,
    ) -> (Rc<RefCell<Control>>, usize) {
        let tab_id = self.header.get_last_id();
        let (untyped, typed) = GuiSystem::create_rc_by_control(control);
        let button = Button::new(
            SizeConstraints(
                SizeConstraint::fixed(width),
                SizeConstraint::fixed(self.height),
            ),
            caption.clone(),
            self.font.clone(),
        );

        self.header
            .add_button_with_close_callback(button, move || close_callback(tab_id));

        self.children.push(untyped);
        (typed, tab_id)
    }

    pub fn add_tab<Control: GuiControl>(
        &mut self,
        caption: String,
        width: i32,
        control: Control,
    ) -> Rc<RefCell<Control>> {
        self.add_tab_with_id(caption, width, control, |_: usize| {})
            .0
    }

    fn hide_selected_tab(&self) {
        if self.base.visible {
            if let Some(selected_tab) = self.get_selected_tab() {
                selected_tab.borrow_mut().on_message(GuiMessage::Hide);
            }
        }
    }

    fn show_selected_tab(&self) {
        if self.base.visible {
            if let Some(selected_tab) = self.get_selected_tab() {
                selected_tab.borrow_mut().on_message(GuiMessage::Show);
            }
        }
    }

    pub fn delete_tab(&mut self, id: usize) {
        if let Some(index) = self.header.find_index(id) {
            self.children.remove(index);
            self.header.delete_button(id);
        }
    }

    pub fn selected_tab_id(&self) -> usize {
        self.header.get_id()
    }

    pub fn get_selected_tab(&self) -> Option<Rc<RefCell<dyn GuiControl>>> {
        self.header
            .find_index(self.selected_tab_id())
            .and_then(|index| self.children.get(index))
            .map(Rc::clone)
    }

    pub fn select_tab(&mut self, id: usize) {
        self.hide_selected_tab();
        self.header.set_id(id);
        self.show_selected_tab();
    }
}

impl GuiControl for TabControl {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        match m {
            GuiMessage::FindDestination(dest, position) => {
                if let Some(selected_tab) = self.get_selected_tab() {
                    if selected_tab
                        .borrow_mut()
                        .get_base_mut()
                        .rect
                        .contains(position)
                    {
                        *dest = GuiSystem::get_child(&selected_tab, position);
                        return true;
                    }
                }

                return self
                    .header
                    .on_message(GuiMessage::FindDestination(dest, position));
            }
            GuiMessage::UpdateSizeConstraints => {
                self.header.on_message(GuiMessage::UpdateSizeConstraints);
                self.base.current_size_constraints =
                    self.header.get_base_mut().current_size_constraints;
                self.base.minimal_size = self.header.get_base_mut().minimal_size;
                if let Some(selected_tab) = self.get_selected_tab() {
                    selected_tab
                        .borrow_mut()
                        .on_message(GuiMessage::UpdateSizeConstraints);
                    let tab_size_constraints = selected_tab
                        .borrow_mut()
                        .get_base_mut()
                        .current_size_constraints;
                    self.base.current_size_constraints.0.absolute = max(
                        self.base.current_size_constraints.0.absolute,
                        tab_size_constraints.0.absolute,
                    );
                    self.base.current_size_constraints.1.absolute +=
                        tab_size_constraints.1.absolute;
                    self.base.current_size_constraints.1.relative = tab_size_constraints.1.relative;

                    let tab_minimal_size = selected_tab.borrow_mut().get_base_mut().minimal_size;
                    self.base.minimal_size.0 = max(self.base.minimal_size.0, tab_minimal_size.0);
                    self.base.minimal_size.1 += tab_minimal_size.1;
                }

                return true;
            }
            GuiMessage::RectUpdated => {
                let mut header_rect = self.base.rect;
                header_rect.right_bottom.1 = header_rect.left_top.1 + self.height;
                GuiSystem::set_rect(&mut self.header, header_rect);
                if let Some(selected_tab) = self.get_selected_tab() {
                    let mut selected_tab_rect = self.base.rect;
                    selected_tab_rect.left_top.1 = self.base.rect.left_top.1 + self.height;
                    GuiSystem::set_rect(selected_tab.borrow_mut().deref_mut(), selected_tab_rect);
                }
                return true;
            }
            GuiMessage::Draw(buf, theme, force) => {
                if self.base.visible {
                    let need_force = self.base.can_draw(force);
                    self.header
                        .on_message(GuiMessage::Draw(buf, theme, need_force));
                    if let Some(selected_tab) = self.get_selected_tab() {
                        let mut child = selected_tab.borrow_mut();
                        let rect = child.get_base_mut().rect;
                        let mut buf_for_child = buf.window_mut(
                            position_to_image_size(self.base.rect.relative(rect.left_top)),
                            position_to_image_size(self.base.rect.relative(rect.right_bottom)),
                        );
                        child.on_message(GuiMessage::Draw(&mut buf_for_child, theme, need_force));
                    }
                }

                return true;
            }
            GuiMessage::GetHotkeys(hotkey_map, active) => {
                let header_active =
                    active && self.base.visible && self.header.get_base_mut().visible;
                self.header
                    .on_message(GuiMessage::GetHotkeys(hotkey_map, header_active));
                for (index, child) in self.children.iter().enumerate() {
                    let active = active
                        && self.base.visible
                        && child.borrow_mut().get_base_mut().visible
                        && index == self.selected_tab_id();
                    child
                        .borrow_mut()
                        .on_message(GuiMessage::GetHotkeys(hotkey_map, active));
                }
                return false;
            }
            GuiMessage::Create => {
                self.header.on_message(m);
                for child in &self.children {
                    child.borrow_mut().on_message(GuiMessage::Create);
                }
                return true;
            }
            GuiMessage::Destroy => {
                self.header.on_message(m);
                for child in &self.children {
                    child.borrow_mut().on_message(GuiMessage::Destroy);
                }
                return true;
            }
            GuiMessage::Show => {
                self.header.on_message(m);
                if let Some(child) = self.get_selected_tab() {
                    let mut child = child.borrow_mut();
                    if child.get_base_mut().visible {
                        child.on_message(GuiMessage::Show);
                    }
                }
                return true;
            }
            GuiMessage::Hide => {
                self.header.on_message(m);
                if let Some(child) = self.get_selected_tab() {
                    let mut child = child.borrow_mut();
                    if child.get_base_mut().visible {
                        child.on_message(GuiMessage::Hide);
                    }
                }
                return true;
            }
            _ => return false,
        }
    }
}
