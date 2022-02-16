use std::cell::{RefCell};
use std::cmp::{min, max};
use std::rc::{Rc};
use std::ops::{Deref, DerefMut};

use crate::clipboard::Clipboard;
use crate::font::*;
use crate::gui::*;
use crate::image::*;
use crate::keys::*;

#[derive(Debug)]
pub enum ContainerLayout {
  Vertical,
  Horizontal,
}

#[derive(Debug)]
pub struct Container {
  base: GuiControlBase,
  layout: ContainerLayout,
  children: Vec<Rc<RefCell<dyn GuiControl>>>,
  support_compression: bool,
}

impl Container {
  pub fn new(size_constraints: SizeConstraints, layout: ContainerLayout) -> Self {
    Self {
      base: GuiControlBase::new(size_constraints),
      layout,
      children: Default::default(),
      support_compression: false,
    }
  }

  pub fn compressed(mut self) -> Self {
    self.support_compression = true;
    self
  }

  pub fn add_child<Control: GuiControl>(&mut self, control: Control) -> Rc<RefCell<Control>> {
    let (untyped, typed) = GuiSystem::create_rc_by_control(control);
    self.children.push(untyped);
    typed
  }

  pub fn delete_child(&mut self, control: &Rc<RefCell<dyn GuiControl>>) {
    let control_base = control.borrow_mut().get_base_mut() as *const _;
    self.children.retain(|child| {
      let child = child.borrow_mut().get_base_mut() as *const _;
      control_base != child
    });
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
    let relative_remainder = if size > sum_absolute { size - sum_absolute } else { 0 };
    let sum_absolute = max(max(sum_absolute, size), 1);

    let mut current_shift = 0;
    let mut sum_child_absolute = 0;
    let mut sum_child_relative = 0;

    for child in &$self.children {
      let mut child = child.borrow_mut();
      let child_size_constraints = child.get_base_mut().current_size_constraints;
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
        if self.support_compression {
          return true;
        }

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

        self.base.current_size_constraints.0.absolute = max(self.base.size_constraints.0.absolute, minimal_size.0);
        self.base.current_size_constraints.1.absolute = max(self.base.size_constraints.1.absolute, minimal_size.1);
        *size_constraints = self.base.current_size_constraints;
        return true;
      }
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
      }
      GuiMessage::Draw(buf, theme) => {
        if self.base.visible {
          for child in &self.children {
            let mut child = child.borrow_mut();
            let rect = child.get_base_mut().rect;
            let mut buf_for_child = buf.window_mut(
              position_to_image_size(self.base.rect.relative(rect.left_top)),
              position_to_image_size(self.base.rect.relative(rect.right_bottom))
            );
            child.on_message(GuiMessage::Draw(&mut buf_for_child, theme));
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
}

impl ColorBox {
  pub fn new(size_constraints: SizeConstraints) -> Self {
    Self {
      base: GuiControlBase::new(size_constraints),
    }
  }
}

impl GuiControl for ColorBox {
  fn get_base_mut(&mut self) -> &mut GuiControlBase {
    &mut self.base
  }

  fn on_message(&mut self, m: GuiMessage) -> bool {
    match m {
      GuiMessage::Draw(buf, theme) => {
        if self.base.visible {
          buf.fill(|d| *d = theme.splitter);
        }

        return true;
      },
      _ => return false,
    }
  }
}

#[derive(Clone)]
pub struct Callback(Rc<dyn Fn() + 'static>);

impl std::fmt::Debug for Callback {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.pad("Callback")
  }
}

#[derive(Debug)]
pub struct Button {
  base: GuiControlBase,
  holded_when_pushed: bool,
  font: Font,
  text: String,
  callback: Option<Callback>,
  check_box: bool,
  button_group: Option<bool>,
}

impl Button {
  pub fn new(size_constraints: SizeConstraints, text: String, font: Font) -> Self {
    Self {
      base: GuiControlBase::new(size_constraints),
      holded_when_pushed: false,
      font: font.layout_vertical(TextLayoutVertical::MIDDLE).layout_horizontal(TextLayoutHorizontal::LEFT),
      text,
      callback: None,
      check_box: false,
      button_group: None,
    }
  }

  pub fn set_callback(&mut self, callback: impl Fn() + 'static) {
    self.callback = Some(Callback(Rc::new(callback)));
  }

  pub fn callback(mut self, callback: impl Fn() + 'static) -> Self {
    self.set_callback(callback);
    self
  }

  pub fn check_box(mut self) -> Self {
    self.check_box = true;
    self
  }
}

impl GuiControl for Button {
  fn get_base_mut(&mut self) -> &mut GuiControlBase {
    &mut self.base
  }

  fn on_message(&mut self, m: GuiMessage) -> bool {
    match m {
      GuiMessage::Draw(buf, theme) => {
        if self.base.visible {
          let size = buf.get_size();
          if size.0 > 0 && size.1 > 0 {
            if self.base.checked {
              buf.fill(|d| *d = avg_color(theme.selected, *d));
            }

            if self.base.pressed {
              buf.fill(|d| *d = avg_color(theme.pressed, *d));
            } else if self.base.highlight {
              buf.fill(|d| *d = avg_color(theme.highlight, *d));
            }

            let check_symbol = if self.button_group.is_some() {"●"} else if self.check_box {"V"} else {""};
            let check_width = min(self.font.get_size(check_symbol).0 * 2, buf.get_size().0);
            let mut caption_dst = buf.window_mut((check_width, 0), buf.get_size());
            let with_checks = self.button_group.is_some() || self.check_box;

            let mut caption_position = (if with_checks {0} else {(caption_dst.get_size().0 - check_width) as i32 / 2}, caption_dst.get_size().1 as i32 / 2);
            if self.base.pressed && self.holded_when_pushed {
              caption_position.0 += 1;
              caption_position.1 += 1;
            }

            let font = if with_checks { self.font.clone() } else { self.font.layout_horizontal(TextLayoutHorizontal::MIDDLE) };
            font.color(theme.font).draw(&self.text, caption_position, &mut caption_dst);

            let check_symbol = if self.button_group.is_some() {
              if self.base.checked {"●"} else {"○"}
            } else if self.check_box {
              if self.base.checked {"V"} else {"□"}
            } else {
              ""
            };

            let mut check_dst = buf.window_mut((0, 0), (check_width, buf.get_size().1));
            let check_position = (check_dst.get_size().0 as i32 / 2, check_dst.get_size().1 as i32 / 2);
            self.font.color(theme.font).layout_horizontal(TextLayoutHorizontal::MIDDLE).draw(check_symbol, check_position, &mut check_dst);
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
        if self.base.rect.contains(position) {
          if self.check_box {
            self.base.checked = !self.base.checked;
          }
          if let Some(Callback(callback)) = &self.callback {
            if let Some(job_system) = job_system.upgrade() {
              job_system.borrow_mut().add_callback(callback.clone());
            }
          }
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
      GuiMessage::Draw(buf, theme) => {
        if self.base.visible {
          let (x, y) = buf.get_size();
          if x > 2 && y > 2 {
            let border_color = theme.font;
            buf.window_mut((0, 0), (x, 1)).fill(|p| *p = border_color);
            buf.window_mut((0, y - 1), (x, y)).fill(|p| *p = border_color);
            buf.window_mut((0, 1), (1, y - 1)).fill(|p| *p = border_color);
            buf.window_mut((x - 1, 1), (x, y - 1)).fill(|p| *p = border_color);
            let mut inner = buf.window_mut((1, 1), (x - 1, y - 1));

            if self.base.focus {
              inner.fill(|d| *d = avg_color(*d, theme.edit_focused));

              let text_before_cursor: String = self
                .text[self.scroll_position as usize .. self.cursor_position as usize]
                .iter()
                .collect();
              let cursor_position = self.font.get_size(&text_before_cursor).0;
              if cursor_position + 2 <= x - 2 {
                inner.window_mut((cursor_position, 0), (cursor_position + 2, y - 2)).fill(|p| *p = border_color);
              }
            } else if self.base.highlight {
              inner.fill(|d| *d = avg_color(*d, theme.edit_highlight));
            } else {
              inner.fill(|d| *d = avg_color(*d, theme.inactive));
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

            self.font.color(theme.font).draw(&visible_text, (1, (y / 2) as i32), &mut inner);
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
          let new_symbol_width = self.font.get_size(self.text[char_index as usize].encode_utf8(&mut buffer)).0 as i32;
          if width_before_mouse + new_symbol_width / 2 > cursor_x {
            break;
          }
          char_index += 1;
          width_before_mouse += new_symbol_width;
        }
        self.cursor_position = char_index;
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
        } else if c == '\x16' { // CTRL+V
          return self.paste();
        }
        return false;
      }
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
      }
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
      GuiMessage::Draw(buf, theme) => {
        if self.base.visible {
          self.scroll_range = max(1, self.scroll_range);
          self.content_size = max(1, min(self.content_size, self.scroll_range));
          self.scroll_position = max(0, min(self.scroll_range - self.content_size, self.scroll_position));

          if self.content_size >= self.scroll_range {
            return true;
          }

          if self.very_small() {
            buf.fill(|d| *d = avg_color(*d, theme.inactive));
            return true;
          }

          let (button_size, button_shift) = self.get_button_size_and_shift();

          macro_rules! top_scroll {
            () => {buf.window_mut((0, 0), (buf.get_size().0, button_shift as usize))};
          }

          macro_rules! button_scroll {
            () => {buf.window_mut((0, button_shift as usize), (buf.get_size().0, (button_size + button_shift) as usize))};
          }

          macro_rules! bottom_scroll {
            () => {buf.window_mut((0, (button_size + button_shift) as usize), buf.get_size())};
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
        self.scroll_position = max(0, min(self.scroll_range - self.content_size, self.scroll_position + delta));
        return old_scroll_position != self.scroll_position;
      }
      GuiMessage::MouseDown(position) => {
        self.set_in_button(position);
        match self.scroll_state {
          ScrollState::Less => self.scroll_position -= self.content_size,
          ScrollState::Greater => self.scroll_position += self.content_size,
          _ => {}
        }

        self.scroll_position = max(0, min(self.scroll_range - self.content_size, self.scroll_position));
        return true;
      }
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
          return false;
        }
      }
      GuiMessage::MouseUp(_, _) => {
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
        scroll_rect.left_top.0 = max(scroll_rect.left_top.0, scroll_rect.right_bottom.0 - self.scroll_width);
        return true;
      }
      GuiMessage::Draw(buf, theme) => {
        self.scroll.base.highlight = self.base.highlight;
        self.scroll.base.focus = self.base.focus;
        self.scroll.base.pressed = self.base.pressed;
        let total_height = self.base.rect.right_bottom.1 - self.base.rect.left_top.1;
        let item_count = (total_height + self.get_item_height() / 2) / max(self.get_item_height(), 1);

        self.scroll.content_size = item_count - 1;
        self.scroll.scroll_range = self.lines.len() as i32;
        let scroll_rect = self.scroll.base.rect;
        let mut buf_for_child = buf.window_mut(
          position_to_image_size(self.base.rect.relative(scroll_rect.left_top)),
          position_to_image_size(self.base.rect.relative(scroll_rect.right_bottom))
        );
        let scroll_result = self.scroll.on_message(GuiMessage::Draw(&mut buf_for_child, theme));

        let first_line = min(max(0, self.scroll.scroll_position) as usize, self.lines.len());
        let last_line = min(first_line + (item_count + 1) as usize, self.lines.len());
        let mut position = (0, 0);
        for line in self.lines[first_line .. last_line].iter() {
          self.font.color(theme.font).draw(&line, position, buf);
          position.1 += self.get_item_height();
        }

        return scroll_result;
      }
      _ => return self.scroll.on_message(m),
    }
  }
}

#[derive(Debug)]
struct TabControlItem {
  caption: String,
  width: i32,
  button: Rc<RefCell<Button>>,
  container: Rc<RefCell<dyn GuiControl>>,
}

#[derive(Debug)]
pub struct TabControl {
  base: GuiControlBase,
  height: i32,
  children: Vec<TabControlItem>,
  selected_tab_index: Rc<RefCell<usize>>,
  font: Font,
}

impl TabControl {
  pub fn new(height: i32, font: Font) -> Self {
    Self {
      base: GuiControlBase::new(SizeConstraints(
          SizeConstraint::flexible(0),
          SizeConstraint::fixed(height + 1),
      )),
      height,
      children: vec![],
      selected_tab_index: Rc::new(RefCell::new(0)),
      font,
    }
  }

  pub fn add_tab<Control: GuiControl>(&mut self, caption: String, width: i32, control: Control) -> Rc<RefCell<Control>> {
    let (untyped, typed) = GuiSystem::create_rc_by_control(control);
    let new_index = self.children.len();
    let tab_index_capture = Rc::downgrade(&self.selected_tab_index);
    let button = Button::new(
      SizeConstraints(
        SizeConstraint::fixed(width),
        SizeConstraint::fixed(self.height),
      ),
      caption.clone(), 
      self.font.clone(),
    ).callback(move || {
      tab_index_capture.upgrade().map(|p| *p.borrow_mut() = new_index);
    });
    self.children.push(TabControlItem{
      caption,
      width,
      button: Rc::new(RefCell::new(button)),
      container: untyped,
    });
    typed
  }

  pub fn get_selected_tab(&self) -> Option<Rc<RefCell<dyn GuiControl>>> {
    self.children.get(*self.selected_tab_index.borrow().deref()).map(|t| t.container.clone())
  }
}

impl GuiControl for TabControl {
  fn get_base_mut(&mut self) -> &mut GuiControlBase {
    &mut self.base
  }

  fn on_message(&mut self, m: GuiMessage) -> bool {
    match m {
      GuiMessage::FindDestination(dest, position) => {
        if let Some(t) = self.children.iter().find(|t|
          t.button.borrow_mut().get_base_mut().rect.contains(position)
        ) {
          let child_as_dyn: Rc<RefCell<dyn GuiControl + 'static>> = t.button.clone();
          *dest = GuiSystem::get_child(&child_as_dyn, position);
        }

        if let Some(selected_tab) = self.get_selected_tab() {
          if selected_tab.borrow_mut().get_base_mut().rect.contains(position) {
            *dest = GuiSystem::get_child(&selected_tab, position);
          }
        }

        return true;
      }
      GuiMessage::UpdateSizeConstraints(size_constraints) => {
        if let Some(selected_tab) = self.get_selected_tab() {
          self.base.current_size_constraints = GuiSystem::get_size_constraints(selected_tab.borrow_mut().deref_mut());
          self.base.current_size_constraints.1.absolute += 1 + self.height;
        } else {
          self.base.current_size_constraints = SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(1 + self.height),
          )
        }

        self.base.current_size_constraints.0.absolute = max(
          self.base.current_size_constraints.0.absolute, 
          self.children.iter().fold(0, |acc, t| acc + t.width)
        );

        *size_constraints = self.base.current_size_constraints;
        return true;
      }
      GuiMessage::RectUpdated => {
        let rect = self.base.rect;
        let size = (rect.right_bottom.0 - rect.left_top.0, rect.right_bottom.1 - rect.left_top.1);
        if self.children.len() > 0 {
          if size.0 == 0 {
            let button_rect = Rect { left_top: (0, 0), right_bottom: (0, 0)};
            for t in &self.children {
              GuiSystem::set_rect(t.button.borrow_mut().deref_mut(), button_rect);
            }
          } else {
            let sum_width = self.children.iter().fold(0, |acc, t| acc + t.width);
            let real_width = min(size.0, sum_width);
            let mut button_rect = Rect { left_top: (0, 0), right_bottom: (0, self.height)};
            let mut current_sum_width = 0;
            for t in &self.children {
              button_rect.left_top.0 = button_rect.right_bottom.0;
              current_sum_width += t.width;
              button_rect.right_bottom.0 = current_sum_width * real_width / sum_width;
              GuiSystem::set_rect(t.button.borrow_mut().deref_mut(), button_rect);
            }
          }

        }
        if let Some(selected_tab) = self.get_selected_tab() {
          let mut selected_tab_rect = rect;
          selected_tab_rect.left_top.1 = self.height + 1;
          GuiSystem::set_rect(selected_tab.borrow_mut().deref_mut(), selected_tab_rect);
        }
        return true;
      }
      GuiMessage::Draw(buf, theme) => {
        if self.base.visible {
          let mut last_rect = 0;
          for (i, t) in self.children.iter().enumerate() {
            let mut child = t.button.borrow_mut();
            let rect = child.get_base_mut().rect;
            last_rect = max(last_rect, rect.right_bottom.0);
            if i == *self.selected_tab_index.borrow() {
              let mut rect_for_select = rect;
              rect_for_select.right_bottom.1 += 1;
              let mut buf_for_select = buf.window_mut(
                position_to_image_size(self.base.rect.relative(rect_for_select.left_top)),
                position_to_image_size(self.base.rect.relative(rect_for_select.right_bottom))
              );
              buf_for_select.fill(|d| *d = avg_color(theme.selected, *d));
            } else {
              let mut rect_for_select = rect;
              rect_for_select.left_top.1 = rect_for_select.right_bottom.1;
              rect_for_select.right_bottom.1 += 1;
              let mut buf_for_select = buf.window_mut(
                position_to_image_size(self.base.rect.relative(rect_for_select.left_top)),
                position_to_image_size(self.base.rect.relative(rect_for_select.right_bottom))
              );
              buf_for_select.fill(|d| *d = theme.splitter);
            }

            let mut buf_for_child = buf.window_mut(
              position_to_image_size(self.base.rect.relative(rect.left_top)),
              position_to_image_size(self.base.rect.relative(rect.right_bottom))
            );

            child.on_message(GuiMessage::Draw(&mut buf_for_child, theme));
          }

          let rect_for_select = Rect{
            left_top: (last_rect, self.height),
            right_bottom: (self.base.rect.right_bottom.0, self.height + 1)
          };
          let mut buf_for_select = buf.window_mut(
            position_to_image_size(self.base.rect.relative(rect_for_select.left_top)),
            position_to_image_size(self.base.rect.relative(rect_for_select.right_bottom))
          );
          buf_for_select.fill(|d| *d = theme.splitter);

          if let Some(selected_tab) = self.get_selected_tab() {
            let mut child = selected_tab.borrow_mut();
            let rect = child.get_base_mut().rect;
            let mut buf_for_child = buf.window_mut(
              position_to_image_size(self.base.rect.relative(rect.left_top)),
              position_to_image_size(self.base.rect.relative(rect.right_bottom))
            );
            child.on_message(GuiMessage::Draw(&mut buf_for_child, theme));
          }
        }
        return true;
      },
      _ => return false,
    }
  }
}
