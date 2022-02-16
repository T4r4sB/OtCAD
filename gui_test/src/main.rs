use application::draw_context::*;
use application::font::*;
use application::gui::*;
use application::gui::gui_components::*;

use curves::curves::*;
use curves::points::*;
use curves::render::*;

mod top_panel;
mod bottom_panel;

mod file_menu;
mod draw_menu;
mod group_menu;
mod transform_menu;
mod options_menu;

use top_panel::*;
use bottom_panel::*;

#[derive(Debug)]
pub struct CadView {
  base: GuiControlBase,
}

impl CadView {
  pub fn new(size_constraints: SizeConstraints,) -> Self {
    Self {
      base: GuiControlBase::new(size_constraints),
    }
  }
}

impl GuiControl for CadView {
  fn get_base_mut(&mut self) -> &mut GuiControlBase {
    &mut self.base
  }

  fn on_message(&mut self, m: GuiMessage) -> bool {
    match m {
      GuiMessage::Draw(buf, _) => {
        if self.base.visible {
          let curve = Curve::<f32>::circle(Point::new(100.0, 100.0), 50.0);
          let e = Entity::Curve(curve);
          let mut buffer = vec![(0, 0); buf.get_size().1 * 4];
          draw_curve(buf, &e, 0, 1.0, &mut buffer, 4);
        }

        return true;
      },
      _ => return false,
    }
  }
}

#[derive(Default)]
struct GuiTest {
}

impl window::Application for GuiTest {
  fn on_create(
    &mut self,
    context: &mut window::Context,
  ) {
    let default_font = context.font_factory.new_font("MS Sans Serif", 15, FontAliasingMode::TT);
    let root = context.gui_system.set_root(Container::new(
      SizeConstraints(
        SizeConstraint::flexible(0),
        SizeConstraint::flexible(0),
      ),
      ContainerLayout::Vertical,
    ));

    create_top_panel(&mut root.borrow_mut(), &default_font);
    let _hr = root.borrow_mut().add_child(ColorBox::new(SizeConstraints(
      SizeConstraint::flexible(0),
      SizeConstraint::fixed(1),
    )));
  
    let _middle = root.borrow_mut().add_child(CadView::new(SizeConstraints(
      SizeConstraint::flexible(100),
      SizeConstraint::flexible(100),
    )));

    let _hr = root.borrow_mut().add_child(ColorBox::new(SizeConstraints(
      SizeConstraint::flexible(0),
      SizeConstraint::fixed(1),
    )));
  
    create_bottom_panel(&mut root.borrow_mut(), &default_font);
  }

  fn on_draw(
    &self,
    draw_context: &mut DrawContext,
  ) {
    draw_context.buffer.fill(|p| {
      *p = 0xFFFFFF;
    });
  }
}

fn main() {
  if let Err(_) = window::run_application("ОТКАД", Box::new(GuiTest::default())) {
    // Do nothing, read message and exit
  }
}
