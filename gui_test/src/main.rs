use application::clipboard::*;
use application::draw_context::*;
use application::font::*;
use application::gui::*;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Default)]
struct GuiTest {
}

impl window::Application for GuiTest {
  fn on_create(
    &mut self,
    gui_system: &mut GuiSystem,
    _clipboard: Rc<RefCell<dyn Clipboard>>, 
    font_factory: &mut FontFactory,
  ) {
    let default_font = font_factory.new_font("Lucida Console", 14);
    let hr_color = 0;

    let root = gui_system.set_root(Container::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::flexible(0),
      ),
      ContainerLayout::Vertical,
    ));

    let _top_panel = root.borrow_mut().add_child(Container::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      ContainerLayout::Horizontal,
    ));

    let _hr = root.borrow_mut().add_child(ColorBox::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(1),
      ),
      hr_color,
    ));

    let middle = root.borrow_mut().add_child(Container::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::flexible(100),
      ),
      ContainerLayout::Horizontal,
    ));

    let _hr = root.borrow_mut().add_child(ColorBox::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(1),
      ),
      hr_color,
    ));

    let bottom_panel = root.borrow_mut().add_child(Container::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(80),
      ),
      ContainerLayout::Horizontal,
    ));

    let left_panel = middle.borrow_mut().add_child(Container::new(
      SizeConstraints(
        SizeConstraint::fixed(100),
        SizeConstraint::flexible(0),
      ),
      ContainerLayout::Vertical,
    ));

    let _hr = middle.borrow_mut().add_child(ColorBox::new(
      SizeConstraints(
        SizeConstraint::fixed(1),
        SizeConstraint::flexible(0),
      ),
      hr_color,
    ));

    let _center = middle.borrow_mut().add_child(Container::new(
      SizeConstraints(
        SizeConstraint::flexible(200),
        SizeConstraint::flexible(0),
      ),
      ContainerLayout::Vertical,
    ));

    let _hr = middle.borrow_mut().add_child(ColorBox::new(
      SizeConstraints(
        SizeConstraint::fixed(1),
        SizeConstraint::flexible(0),
      ),
      hr_color,
    ));

    let right_panel = middle.borrow_mut().add_child(Container::new(
      SizeConstraints(
        SizeConstraint::fixed(150),
        SizeConstraint::flexible(0),
      ),
      ContainerLayout::Vertical,
    ));

    let _line = left_panel.borrow_mut().add_child(Button::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Линия".to_string(),
      default_font.clone(),
    ));

    let _circle = left_panel.borrow_mut().add_child(Button::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Окружность".to_string(),
      default_font.clone(),
    ));

    let _arc = left_panel.borrow_mut().add_child(Button::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Дуга".to_string(),
      default_font.clone(),
    ));

    let _hr = left_panel.borrow_mut().add_child(ColorBox::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(1),
      ),
      hr_color,
    ));

    let _enlarge = left_panel.borrow_mut().add_child(Button::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Удлинить".to_string(),
      default_font.clone(),
    ));

    let _cut = left_panel.borrow_mut().add_child(Button::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Обрезать".to_string(),
      default_font.clone(),
    ));

    let _hr = left_panel.borrow_mut().add_child(ColorBox::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(1),
      ),
      hr_color,
    ));

    let _move = left_panel.borrow_mut().add_child(Button::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Переместить".to_string(),
      default_font.clone(),
    ));

    let _rotate = left_panel.borrow_mut().add_child(Button::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Повернуть".to_string(),
      default_font.clone(),
    ));

    let _copy = left_panel.borrow_mut().add_child(Button::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Скопировать".to_string(),
      default_font.clone(),
    ));

    let _grid = right_panel.borrow_mut().add_child(CheckBox::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Показать сетку".to_string(),
      default_font.clone(),
    ));

    let _hr = right_panel.borrow_mut().add_child(ColorBox::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(1),
      ),
      hr_color,
    ));

    let _vertex = right_panel.borrow_mut().add_child(CheckBox::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Вершины".to_string(),
      default_font.clone(),
    ));

    let _coord = right_panel.borrow_mut().add_child(CheckBox::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Узлы сетки".to_string(),
      default_font.clone(),
    ));

    let _intersections = right_panel.borrow_mut().add_child(CheckBox::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Пересечения".to_string(),
      default_font.clone(),
    ));

    let _centers = right_panel.borrow_mut().add_child(CheckBox::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::fixed(20),
      ),
      "Центры".to_string(),
      default_font.clone(),
    ));

    let sv = bottom_panel.borrow_mut().add_child(ListBox::new(
      SizeConstraints(
        SizeConstraint::flexible(160),
        SizeConstraint::flexible(0),
      ),
      16,
      default_font.clone(),
    ));

    sv.borrow_mut().lines.push("Lorem".to_string());
    sv.borrow_mut().lines.push("Ipsum".to_string());
    sv.borrow_mut().lines.push("Lorem Ipsum".to_string());
    sv.borrow_mut().lines.push("Я пишу по-русски".to_string());
    sv.borrow_mut().lines.push("Ещё линия".to_string());
    sv.borrow_mut().lines.push("Ещё линия очень очень большой длины чтоб посмотреть что получится из этого".to_string());
  }

  fn on_draw(
    &mut self,
    draw_context: &mut DrawContext,
  ) {
    draw_context.buffer.fill(|p| *p = 0xDDBB99);
  }
}

fn main() {
  if let Err(_) = window::run_application(Box::new(GuiTest::default())) {
    // Do nothing, read message and exit
  }
}
