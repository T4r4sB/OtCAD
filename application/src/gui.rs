use crate::image::*;
use crate::draw_context::*;
use crate::clipboard::Clipboard;
use crate::font::*;
use crate::keys::*;

use std::cell::{RefCell};
use std::cmp::{min, max};
use std::rc::Rc;
use std::ops::DerefMut;

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
  left_top: Position,
  right_bottom: Position,
}

impl Rect {
  fn contains(self, position: Position) -> bool {
    self.left_top.0 <= position.0 &&
    self.left_top.1 <= position.1 &&
    self.right_bottom.0 > position.0 &&
    self.right_bottom.1 > position.1
  }

  fn relative(self, position: Position) -> Position {
    (
      position.0 - self.left_top.0,
      position.1 - self.left_top.1
    )
  }
}

#[derive(Debug)]
pub struct GuiControlBase {
  size_constraints: SizeConstraints,
  visible: bool,
  focus: bool,
  highlight: bool,
  pressed: bool,
  rect: Rect,
}

impl GuiControlBase {
  fn new(size_constraints: SizeConstraints) -> Self {
    Self {
      size_constraints,
      visible: true,
      focus: false,
      highlight: false,
      pressed: false,
      rect: Rect::default(),
    }
  }
}

pub enum GuiMessage<'i, 'j> {
  Draw(&'i mut DrawContext<'j>),
  UpdateSizeConstraints(&'i mut SizeConstraints),
  FindDestination(&'i mut Rc<RefCell<dyn GuiControl>>, Position),
  RectUpdated,
  MouseDown(Position),
  MouseMove(Position),
  MouseUp(Position),
  MouseWheel(Position, i32),
  Char(char),
  KeyDown(Key),
  KeyUp(Key),
}

pub trait GuiControl: std::fmt::Debug + 'static {
  fn get_base_mut(&mut self) -> &mut GuiControlBase;
  fn on_message(&mut self, m: GuiMessage) -> bool;
}

pub struct GuiSystem {
  root: Option<Rc<RefCell<dyn GuiControl>>>,
  focus: Option<Rc<RefCell<dyn GuiControl>>>,
  highlight: Option<Rc<RefCell<dyn GuiControl>>>,
  pressed: Option<Rc<RefCell<dyn GuiControl>>>,

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
      let mut new = new_ptr.borrow_mut();
      let new_base = new.get_base_mut();
      if new_base.$field {
        return false;
      }

      new_base.$field = true;
      drop(new);
      off_old_flag();
      $self.$field = Some(new_ptr);
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
  pub fn new() -> Self {
    Self {
      root: None,
      focus: None,
      highlight: None,
      pressed: None,
      updated: false,
    }
  }

  fn set_rect(control: &mut dyn GuiControl, rect: Rect) {
    assert!(rect.right_bottom.0 >= rect.left_top.0);
    assert!(rect.right_bottom.1 >= rect.left_top.1);
    control.get_base_mut().rect = rect;
    control.on_message(GuiMessage::RectUpdated);
  }

  fn get_child(control: &Rc<RefCell<dyn GuiControl>>, position: Position) -> Rc<RefCell<dyn GuiControl>> {
    let mut result = control.clone();
    control.borrow_mut().on_message(GuiMessage::FindDestination(&mut result, position));
    result
  }

  fn get_size_constraints(control: &mut dyn GuiControl) -> SizeConstraints {
    let mut size_constraints = control.get_base_mut().size_constraints;
    control.on_message(GuiMessage::UpdateSizeConstraints(&mut size_constraints));
    size_constraints
  }

  fn get_focus(&self) -> Option<Rc<RefCell<dyn GuiControl>>> {
    self.focus.clone()
  }

  fn set_focus(&mut self, new_focus: Option<Rc<RefCell<dyn GuiControl>>>) -> bool {
    set_property!(self, new_focus, get_focus, focus);
  }

  fn get_highlight(&self) -> Option<Rc<RefCell<dyn GuiControl>>> {
    self.highlight.clone()
  }

  fn set_highlight(&mut self, new_highlight: Option<Rc<RefCell<dyn GuiControl>>>) -> bool {
    set_property!(self, new_highlight, get_highlight, highlight);
  }

  fn get_pressed(&self) -> Option<Rc<RefCell<dyn GuiControl>>> {
    self.pressed.clone()
  }

  fn set_pressed(&mut self, new_pressed: Option<Rc<RefCell<dyn GuiControl>>>) -> bool {
    set_property!(self, new_pressed, get_pressed, pressed);
  }

  pub fn on_draw(&mut self, draw_context: &mut DrawContext) {
    if let Some(root) = &self.root {
      let mut root = root.borrow_mut();
      if !self.updated {
        Self::set_rect(root.deref_mut(), Rect{
          left_top: (0, 0), 
          right_bottom: image_size_to_position(draw_context.buffer.get_size()),
        });
        self.updated = true;
      }
      root.on_message(GuiMessage::Draw(draw_context));
    }
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
      if child.borrow_mut().on_message(GuiMessage::MouseDown(position)) {
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
      let handler = if let Some(pressed) = &maybe_pressed {pressed.clone()} else {Self::get_child(&root, position)};
      let handled = handler.borrow_mut().on_message(GuiMessage::MouseMove(position));
      let changed_highlight = self.set_highlight(Some(handler));
      return handled || changed_highlight;
    }

    return false;
  }

  pub fn on_mouse_wheel(&mut self, position: Position, delta: i32) -> bool {
    if let Some(root) = &self.root {
      let maybe_pressed = self.get_pressed();
      let handler = if let Some(pressed) = &maybe_pressed {pressed.clone()} else {Self::get_child(&root, position)};
      return handler.borrow_mut().on_message(GuiMessage::MouseWheel(position, delta));
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
      let handler = if let Some(pressed) = &maybe_pressed {pressed.clone()} else {Self::get_child(&root, position)};
      if handler.borrow_mut().on_message(GuiMessage::MouseUp(position)) {
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
      return focus.borrow_mut().on_message(GuiMessage::KeyDown(k));
    }

    return false;
  }

  pub fn on_key_up(&mut self, k: Key) -> bool {
    if let Some(focus) = self.get_focus() {
      return focus.borrow_mut().on_message(GuiMessage::KeyUp(k));
    }

    return false;
  }

  pub fn set_root<Control: GuiControl>(&mut self, control: Control) -> Rc<RefCell<Control>> {
    let result = Rc::new(RefCell::new(control));
    self.root = Some(result.clone());
    result
  }
}

#[derive(Debug)]
pub enum ContainerLayout {
  Vertical,
  Horizontal,
}

#[derive(Debug)]
pub struct Container {
  base: GuiControlBase,
  real_size_constraints: SizeConstraints,
  layout: ContainerLayout,
  children: Vec<Rc<RefCell<dyn GuiControl>>>,
}

impl Container {
   pub fn new(size_constraints: SizeConstraints, layout: ContainerLayout) -> Self {
     Self {
       base: GuiControlBase::new(size_constraints),
       real_size_constraints: size_constraints,
       layout,
       children: Default::default(),
     }
   }

   pub fn add_child<Control: GuiControl>(&mut self, control: Control) -> Rc<RefCell<Control>> {
     let result = Rc::new(RefCell::new(control));
     self.children.push(result.clone());
     result
   }
}

macro_rules! set_layout {
  ($self: expr, $index0: tt, $index1: tt) => {
    let mut sum_relative = 0;
    let mut sum_absolute = 0;
    for child in &$self.children {
      let child_size_constraints = child.borrow_mut().get_base_mut().size_constraints;
      sum_relative += child_size_constraints.$index1.relative;
      sum_absolute += child_size_constraints.$index1.absolute;
    }
    let rect = $self.base.rect;
    let size = rect.right_bottom.$index1 - rect.left_top.$index1;
    let perp_size = rect.right_bottom.$index0 - rect.left_top.$index0;
    let sum_relative = max(max(sum_relative, 100), 1);
    let relative_remainder = if size > sum_absolute { size - sum_absolute } else { 0 };
    let sum_absolute = max(max(sum_absolute, size), 1);

    let mut current_shift = 0;
    let mut sum_child_absolute = 0;
    let mut sum_child_relative = 0;

    for child in &$self.children {
      let mut child = child.borrow_mut();
      let child_size_constraints = child.get_base_mut().size_constraints;
      sum_child_absolute += child_size_constraints.$index1.absolute;
      sum_child_relative += child_size_constraints.$index1.relative;

      let next_shift =
        sum_child_absolute * size / sum_absolute +
        sum_child_relative * relative_remainder / sum_relative;
      let mut child_rect = Rect::default();
      child_rect.left_top.$index0 = rect.left_top.$index0;
      child_rect.left_top.$index1 = rect.left_top.$index1 + current_shift;
      child_rect.right_bottom.$index0 = min(
        rect.left_top.$index0
        + child_size_constraints.$index0.absolute
        + (perp_size - child_size_constraints.$index0.absolute)
        * child_size_constraints.$index0.relative / 100,
        rect.right_bottom.$index0
      );
      child_rect.right_bottom.$index1 = rect.left_top.$index1 + next_shift;
      GuiSystem::set_rect(child.deref_mut(), child_rect);
      current_shift = next_shift;
    }
  };
}

impl GuiControl for Container {
  fn get_base_mut(&mut self) -> &mut GuiControlBase {
    &mut self.base
  }

  fn on_message(&mut self, m: GuiMessage) -> bool {
    match m {
      GuiMessage::FindDestination(dest, position) => {
        if let Some(child) = self.children.iter().find(|child|
          child.borrow_mut().get_base_mut().rect.contains(position)
        ) {
          *dest = GuiSystem::get_child(child, position);
        }

        return true;
      }
      GuiMessage::UpdateSizeConstraints(size_constraints) => {
        let child_real_constraints: Vec<_> = self.children.iter_mut().map(|c|
          GuiSystem::get_size_constraints(c.borrow_mut().deref_mut())
        ).collect();

        let mut minimal_size: Position = (0, 0);
        match self.layout {
          ContainerLayout::Vertical => {
            for child_real_constraints in &child_real_constraints {
              minimal_size.0 = max(minimal_size.0, child_real_constraints.0.absolute);
              minimal_size.1 += child_real_constraints.1.absolute;
            }
          },
          ContainerLayout::Horizontal => {
            for child_real_constraints in &child_real_constraints {
              minimal_size.0 += child_real_constraints.0.absolute;
              minimal_size.1 = max(minimal_size.1, child_real_constraints.1.absolute);
            }
          },
        }

        size_constraints.0.absolute = max(size_constraints.0.absolute, minimal_size.0);
        size_constraints.1.absolute = max(size_constraints.1.absolute, minimal_size.1);
        return true;
      },
      GuiMessage::RectUpdated => {
        match self.layout {
          ContainerLayout::Horizontal => {
            set_layout!(self, 1, 0);
          },
          ContainerLayout::Vertical => {
            set_layout!(self, 0, 1);
          }
        }
        return true;
      },
      GuiMessage::Draw(draw_context) => {
        if self.base.visible {
          for child in &self.children {
            let mut child = child.borrow_mut();
            let rect = child.get_base_mut().rect;
            let mut context_for_child = draw_context.window(
              position_to_image_size(self.base.rect.relative(rect.left_top)),
              position_to_image_size(self.base.rect.relative(rect.right_bottom))
            );
            child.on_message(GuiMessage::Draw(&mut context_for_child));
          }
        }
        return true;
      },
      _ => return false,
    }
  }
}

#[derive(Debug)]
pub struct ColorBox {
  base: GuiControlBase,
  color: u32,
}

impl ColorBox {
  pub fn new(size_constraints: SizeConstraints, color: u32) -> Self {
    Self {
      base: GuiControlBase::new(size_constraints),
      color,
    }
  }
}

impl GuiControl for ColorBox {
  fn get_base_mut(&mut self) -> &mut GuiControlBase {
    &mut self.base
  }

  fn on_message(&mut self, m: GuiMessage) -> bool {
    match m {
      GuiMessage::Draw(draw_context) => {
        if self.base.visible {
          let dst = &mut draw_context.buffer;
          dst.fill(|p| *p = (((*p ^ self.color) & 0xFEFEFE) >> 1) + (*p & self.color));
        }
        return true;
      },
      _ => return false,
    }
  }
}

fn _to_black_1_2(p: &mut u32) {*p -= (*p & 0xFEFEFE) >> 1;}
fn to_black_1_4(p: &mut u32) {*p -= (*p & 0xFCFCFC) >> 2;}
fn to_black_1_8(p: &mut u32) {*p -= (*p & 0xF8F8F8) >> 3;}

fn to_white_3_4(p: &mut u32) {*p = ((*p & 0xFCFCFC) >> 2) + 0xC0C0C0;}
fn to_white_1_2(p: &mut u32) {*p += ((0xFFFFFF - *p) & 0xFEFEFE) >> 1;}
fn _to_white_1_4(p: &mut u32) {*p += ((0xFFFFFF - *p) & 0xFCFCFC) >> 2;}
fn _to_white_1_8(p: &mut u32) {*p += ((0xFFFFFF - *p) & 0xF8F8F8) >> 3;}

pub struct Callback(Rc<dyn Fn() + 'static>);

impl std::fmt::Debug for Callback {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.pad("Callbacl")
  }
}

#[derive(Debug)]
pub struct Button {
  base: GuiControlBase,
  holded_when_pushed: bool,
  font: Font,
  text: String,
  callback: Option<Callback>,
}

impl Button {
  pub fn new(size_constraints: SizeConstraints, text: String, font: Font) -> Self {
    Self {
      base: GuiControlBase::new(size_constraints),
      holded_when_pushed: false,
      font: font.layout_vertical(TextLayoutVertical::MIDDLE).layout_horizontal(TextLayoutHorizontal::MIDDLE),
      text,
      callback: None,
    }
  }

  pub fn set_callback(&mut self, callback: impl Fn() + 'static) {
    self.callback = Some(Callback(Rc::new(callback)));
  }

  pub fn callback(mut self, callback: impl Fn() + 'static) -> Self {
    self.set_callback(callback);
    self
  }
}

impl GuiControl for Button {
  fn get_base_mut(&mut self) -> &mut GuiControlBase {
    &mut self.base
  }

  fn on_message(&mut self, m: GuiMessage) -> bool {
    match m {
      GuiMessage::Draw(draw_context) => {
        if self.base.visible {
          let size = draw_context.buffer.get_size();
          if size.0 > 0 && size.1 > 0 {
            let dst = &mut draw_context.buffer;
            if self.base.pressed || self.base.highlight {
              dst.fill(to_black_1_8);
            }

            let mut caption_position = (dst.get_size().0 as i32 / 2, dst.get_size().1 as i32 / 2);
            if self.base.pressed && self.holded_when_pushed {
              caption_position.0 += 1;
              caption_position.1 += 1;
            }
            self.font.draw(&self.text, caption_position, dst);
          }
        }
        return false;
      },
      GuiMessage::MouseDown(_) => {
        self.holded_when_pushed = true;
        return true;
      },
      GuiMessage::MouseMove(position) => {
        if self.base.pressed {
          let prev_holded_when_pushed = self.holded_when_pushed;
          self.holded_when_pushed = self.base.rect.contains(position);
          return self.holded_when_pushed != prev_holded_when_pushed;
        } else {
          return false;
        }
      }
      GuiMessage::MouseUp(position) => { 
        if self.base.rect.contains(position) {
          if let Some(Callback(callback)) = &self.callback {
            callback();
          }
        }
        return true;
      }
      _ => return false,
    }
  }
}

#[derive(Debug)]
pub struct CheckBox {
  base: GuiControlBase,
  holded_when_pushed: bool,
  font: Font,
  text: String,
  checked: bool,
}

impl CheckBox {
  pub fn new(size_constraints: SizeConstraints, text: String, font: Font) -> Self {
    Self {
      base: GuiControlBase::new(size_constraints),
      holded_when_pushed: false,
      font: font.layout_vertical(TextLayoutVertical::MIDDLE).layout_horizontal(TextLayoutHorizontal::MIDDLE),
      text,
      checked: false,
    }
  }
}

impl GuiControl for CheckBox {
  fn get_base_mut(&mut self) -> &mut GuiControlBase {
    &mut self.base
  }

  fn on_message(&mut self, m: GuiMessage) -> bool {
    match m {
      GuiMessage::Draw(draw_context) => {
        if self.base.visible {
          let size = draw_context.buffer.get_size();
          if size.0 > 0 && size.1 > 0 {
            let dst = &mut draw_context.buffer;
            if self.checked {
              dst.fill(to_white_1_2);
            }

            if self.base.pressed || self.base.highlight {
              if self.checked {
                dst.fill(to_white_1_2);
              } else {
                dst.fill(to_black_1_8);
              }
            }

            let check_width = self.font.get_size("V").0 * 2;

            let mut font_dst = dst.window_mut((0, 0), (dst.get_size().0 - check_width, dst.get_size().1));
            let mut caption_position = (font_dst.get_size().0 as i32 / 2, font_dst.get_size().1 as i32 / 2);
            if self.base.pressed && self.holded_when_pushed {
              caption_position.0 += 1;
              caption_position.1 += 1;
            }
            self.font.draw(&self.text, caption_position, &mut font_dst);

            if self.checked {
              let mut check_dst = dst.window_mut((dst.get_size().0 - check_width, 0), dst.get_size());
              let check_position = (check_dst.get_size().0 as i32 / 2, check_dst.get_size().1 as i32 / 2);
              self.font.draw("V", check_position, &mut check_dst);
            }
          }
        }
        return false;
      },
      GuiMessage::MouseDown(_) => {
        self.holded_when_pushed = true;
        return true;
      },
      GuiMessage::MouseMove(position) => {
        if self.base.pressed {
          let prev_holded_when_pushed = self.holded_when_pushed;
          self.holded_when_pushed = self.base.rect.contains(position);
          return self.holded_when_pushed != prev_holded_when_pushed;
        } else {
          return false;
        }
      }
      GuiMessage::MouseUp(position) => { 
        if self.base.rect.contains(position) {
          self.checked = !self.checked;
        }
        return true;
      }
      _ => return false,
    }
  }
}

#[derive(Debug)]
pub struct Edit {
  base: GuiControlBase,

  text: Vec<char>,
  font: Font,
  clipboard: Rc<RefCell<dyn Clipboard>>,
  scroll_position: i32,
  cursor_position: i32,
}

impl Edit {
  pub fn new(size_constraints: SizeConstraints, font: Font, clipboard: Rc<RefCell<dyn Clipboard>>) -> Self {
    Self {
      base: GuiControlBase::new(size_constraints),
      text: Default::default(),
      font: font.layout_vertical(TextLayoutVertical::MIDDLE).layout_horizontal(TextLayoutHorizontal::LEFT),
      clipboard,
      scroll_position: 0,
      cursor_position: 0,
    }
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
      width_before_cursor += self.font.get_size(self.text[(minimal_scroll - 1) as usize].encode_utf8(&mut buffer)).0 as i32;
      if width_before_cursor > width {
        break;
      }

      minimal_scroll -= 1;
    }

    self.scroll_position = minimal_scroll;
  }

  fn char_is_valid(c: char) -> bool {
    match c {
      '\x00' ..= '\x1f' => false,
      _ => return true,
    }
  }

  fn paste(&mut self) -> bool {
    if let Some(text) = self.clipboard.clone().borrow().get_string() {
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
      GuiMessage::Draw(draw_context) => {
        if self.base.visible {
          let dst = &mut draw_context.buffer;
          let (x, y) = dst.get_size();
          if x > 2 && y > 2 {
            dst.window_mut((0, 0), (x, 1)).fill(|p| *p = 0);
            dst.window_mut((0, y - 1), (x, y)).fill(|p| *p = 0);
            dst.window_mut((0, 1), (1, y - 1)).fill(|p| *p = 0);
            dst.window_mut((x - 1, 1), (x, y - 1)).fill(|p| *p = 0);
            let mut inner = dst.window_mut((1, 1), (x - 1, y - 1));

            if self.base.focus {
              inner.fill(|p| *p = 0xFFFFFF);

              let text_before_cursor: String = self
                .text[self.scroll_position as usize .. self.cursor_position as usize]
                .iter()
                .collect();
              let cursor_position = self.font.get_size(&text_before_cursor).0;
              if cursor_position + 2 <= x - 2 {
                inner.window_mut((cursor_position, 0), (cursor_position + 2, y - 2)).fill(|p| *p = 0);
              }
            } else if self.base.highlight {
              inner.fill(to_white_3_4);
            } else {
              inner.fill(to_white_1_2);
            }

            let mut visible_text = String::new();
            let mut width = 0;
            let mut pos = self.scroll_position as usize;
            loop {
              if pos >= self.text.len() {
                break;
              }

              let mut buffer = [0u8; 4];
              width += self.font.get_size(self.text[pos].encode_utf8(&mut buffer)).0;
              visible_text.push(self.text[pos]);
              if width > x - 2 {
                break;
              }
              pos += 1;
            }

            self.font.draw(&visible_text, (1, (y / 2) as i32), &mut inner);
          }
        }

        return true;
      },
      GuiMessage::MouseDown(position) => {
        let mut width_before_mouse = 0;
        let mut char_index = self.scroll_position;
        let mut buffer = [0u8; 4];
        let cursor_x = position.0 - self.base.rect.left_top.0 - 1;
        while char_index < self.text.len() as i32 {
          let new_symbol_width = self.font.get_size(self.text[char_index as usize].encode_utf8(&mut buffer)).0 as i32;
          if width_before_mouse + new_symbol_width / 2 > cursor_x {
            break;
          }
          char_index += 1;
          width_before_mouse += new_symbol_width;
        }
        self.cursor_position = char_index;
        return true;
      },
      GuiMessage::MouseUp(_) => {
        return true;
      },
      GuiMessage::Char(c) => { 
        if Self::char_is_valid(c) {
          self.text.insert(self.cursor_position as usize, c);
          self.cursor_position += 1;
          self.adjust_cursor_position();
          return true;
        } else if c == '\x16' { // CTRL+V
          return self.paste();
        }
        return false;
      },
      GuiMessage::KeyDown(k) => {
        match k {
          Key::Left => if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.adjust_cursor_position();
            return true;
          },
          Key::Right => if self.cursor_position < self.text.len() as i32 {
            self.cursor_position += 1;
            self.adjust_cursor_position();
            return true;
          },
          Key::Delete => if self.cursor_position < self.text.len() as i32 {
            self.text.remove(self.cursor_position as usize);
            self.adjust_cursor_position();
            return true;
          },
          Key::Back => if self.cursor_position > 0 {
            self.text.remove(self.cursor_position as usize - 1);
            self.cursor_position -= 1;
            self.adjust_cursor_position();
            return true;
          },
          Key::Insert => return self.paste(),
          _ => {}
        }
    
        return false;
      },
      _ => return false,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum ScrollState {
  Invalid, Less, OnButton(i32, i32), Greater,
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
    let button_position = self.scroll_position * (height - button_size) / max(1, self.scroll_range - self.content_size);
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
      GuiMessage::Draw(draw_context) => {
        if self.base.visible {
          self.scroll_range = max(1, self.scroll_range);
          self.content_size = max(1, min(self.content_size, self.scroll_range));
          self.scroll_position = max(0, min(self.scroll_range - self.content_size, self.scroll_position));

          if self.content_size >= self.scroll_range {
            return true;
          }

          let dst = &mut draw_context.buffer;

          if self.very_small() {
            dst.fill(to_white_1_2);
            return true;
          }

          let (button_size, button_shift) = self.get_button_size_and_shift();

          macro_rules! top_scroll {
            () => {dst.window_mut((0, 0), (dst.get_size().0, button_shift as usize))};
          }

          macro_rules! button_scroll {
            () => {dst.window_mut((0, button_shift as usize), (dst.get_size().0, (button_size + button_shift) as usize))};
          }

          macro_rules! bottom_scroll {
            () => {dst.window_mut((0, (button_size + button_shift) as usize), dst.get_size())};
          }

          if self.base.pressed {
            match self.scroll_state {
              ScrollState::Less => top_scroll!().fill(|p| *p = 0xFFFFFF),
              _ => top_scroll!().fill(to_white_1_2),
            }
            match self.scroll_state {
              ScrollState::Greater => bottom_scroll!().fill(|p| *p = 0xFFFFFF),
              _ => bottom_scroll!().fill(to_white_1_2),
            }
            button_scroll!().fill(to_black_1_4);
          } else if self.base.highlight {
            match self.scroll_state {
              ScrollState::Less => top_scroll!().fill(to_white_3_4),
              _ => top_scroll!().fill(to_white_1_2),
            }
            match self.scroll_state {
              ScrollState::Greater => bottom_scroll!().fill(to_white_3_4),
              _ => bottom_scroll!().fill(to_white_1_2),
            }
            button_scroll!().fill(to_black_1_8);
          } else {
            top_scroll!().fill(to_white_1_2);
            bottom_scroll!().fill(to_white_1_2);
          }
        }
        return true;
      },
      GuiMessage::MouseWheel(_, delta) => {
        let old_scroll_position = self.scroll_position;
        self.scroll_position = max(0, min(self.scroll_range - self.content_size, self.scroll_position + delta));
        return old_scroll_position != self.scroll_position;
      },
      GuiMessage::MouseDown(position) => {
        self.set_in_button(position);
        match self.scroll_state {
          ScrollState::Less => self.scroll_position -= self.content_size,
          ScrollState::Greater => self.scroll_position += self.content_size,
          _ => {}
        }

        self.scroll_position = max(0, min(self.scroll_range - self.content_size, self.scroll_position));
        return true;
      },
      GuiMessage::MouseMove(position) => {
        if self.base.pressed {
          match self.scroll_state {
            ScrollState::OnButton(yl, yh) => {
              let prev_scroll_position = self.scroll_position;
              let (button_size, button_shift) = self.get_button_size_and_shift(); // before shift
              let free_scroll_size = self.base.rect.right_bottom.1 - self.base.rect.left_top.1 - button_size;
              if yl + yh > 0 && free_scroll_size > 0 {
                let old_position = self.base.rect.left_top.1 + button_shift + (button_size * yl + (yl + yh) / 2) / (yl + yh);
                let screen_delta = position.1 - old_position;
                let delta = (screen_delta * (self.scroll_range - self.content_size) + (if screen_delta >= 0 {1} else {-1}) * free_scroll_size / 2) / max(1, free_scroll_size);
                self.scroll_position += delta;
                self.scroll_position = max(0, min(self.scroll_range - self.content_size, self.scroll_position));
              }
              return prev_scroll_position != self.scroll_position;
            }
            _ => {
              return false;
            }
          } 
        } else {
          let prev_in_button = self.scroll_state;
          self.set_in_button(position);
          return std::mem::discriminant(&prev_in_button) != std::mem::discriminant(&self.scroll_state);
        }
      },
      GuiMessage::MouseUp(_) => {
        return true;
      },
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
      font: font.layout_vertical(TextLayoutVertical::TOP).layout_horizontal(TextLayoutHorizontal::LEFT),
      lines: Default::default(),
    }
  }

  fn get_item_height(&self) -> i32 {
    self.font.get_size("M").1 as i32 * 2 / 2
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
        scroll_rect.left_top.0 = max(scroll_rect.left_top.0, scroll_rect.right_bottom.0 - self.scroll_width);
        return true;
      },
      GuiMessage::Draw(draw_context) => {
        self.scroll.base.highlight = self.base.highlight;
        self.scroll.base.focus = self.base.focus;
        self.scroll.base.pressed = self.base.pressed;
        let total_height = self.base.rect.right_bottom.1 - self.base.rect.left_top.1;
        let item_count = (total_height + self.get_item_height() / 2) / self.get_item_height();

        self.scroll.content_size = item_count - 1;
        self.scroll.scroll_range = self.lines.len() as i32;
        let scroll_rect = self.scroll.base.rect;
        let mut context_for_child = draw_context.window(
          position_to_image_size(self.base.rect.relative(scroll_rect.left_top)),
          position_to_image_size(self.base.rect.relative(scroll_rect.right_bottom))
        );
        let scroll_result = self.scroll.on_message(GuiMessage::Draw(&mut context_for_child));

        let first_line = min(max(0, self.scroll.scroll_position) as usize, self.lines.len());
        let last_line = min(first_line + (item_count + 1) as usize, self.lines.len());
        let mut position = (0, 0);
        for line in self.lines[first_line .. last_line].iter() {
          self.font.draw(&line, position, &mut draw_context.buffer);
          position.1 += self.get_item_height();
        }


        return scroll_result;
      },
      _ => return self.scroll.on_message(m),
    }
  }
}
