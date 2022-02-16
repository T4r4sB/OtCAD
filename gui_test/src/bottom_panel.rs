use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::*;
use application::gui::gui_components::*;

pub fn create_bottom_panel(root: &mut Container, font: &Font) -> Rc<RefCell<Container>> {
  let font_height = font.get_size("8").1 as i32 + 2;

  let bottom_panel = root.add_child(Container::new(SizeConstraints(
    SizeConstraint::flexible(0),
    SizeConstraint::fixed(font_height),
  ), ContainerLayout::Horizontal));

  let grid_caption = "Сетка";
  let _grid_button = bottom_panel.borrow_mut().add_child(Button::new(
    SizeConstraints(
      SizeConstraint::fixed(font.get_size(grid_caption).0 as i32 + font_height),
      SizeConstraint::fixed(font_height),
    ),
    grid_caption.to_string(),
    font.clone(),
  ).check_box());

  let grid_nodes_caption = "Узлы сетки";
  let _grid_nodes_button = bottom_panel.borrow_mut().add_child(Button::new(
    SizeConstraints(
      SizeConstraint::fixed(font.get_size(grid_nodes_caption).0 as i32 + font_height),
      SizeConstraint::fixed(font_height),
    ),
    grid_nodes_caption.to_string(),
    font.clone(),
  ).check_box());

  let endpoints_caption = "Концы";
  let _endpoints_button = bottom_panel.borrow_mut().add_child(Button::new(
    SizeConstraints(
      SizeConstraint::fixed(font.get_size(endpoints_caption).0 as i32 + font_height),
      SizeConstraint::fixed(font_height),
    ),
    endpoints_caption.to_string(),
    font.clone(),
  ).check_box());

  let intersections_caption = "Пересечения";
  let _intersections_button = bottom_panel.borrow_mut().add_child(Button::new(
    SizeConstraints(
      SizeConstraint::fixed(font.get_size(intersections_caption).0 as i32 + font_height),
      SizeConstraint::fixed(font_height),
    ),
    intersections_caption.to_string(),
    font.clone(),
  ).check_box());


  bottom_panel
}
