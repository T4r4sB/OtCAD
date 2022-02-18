use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

pub fn create_draw_menu(parent: &mut TabControl, font: &Font) -> Rc<RefCell<Container>> {
    let font_height = font.get_size("8").1 as i32 + 2;
    let menu_caption = "Рисовать";
    let draw_menu = parent.add_tab(
        menu_caption.to_string(),
        font.get_size(menu_caption).0 as i32 + font_height,
        Container::new(
            SizeConstraints(
                SizeConstraint::flexible(0),
                SizeConstraint::fixed(font_height),
            ),
            ContainerLayout::Horizontal,
        ),
    );

    let line_caption = "Отрезок";
    let _line_button = draw_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(line_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        line_caption.to_string(),
        font.clone(),
    ));

    let circle_caption = "Круг";
    let _circle_button = draw_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(circle_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        circle_caption.to_string(),
        font.clone(),
    ));

    let arc_caption = "Дуга";
    let _arc_button = draw_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(arc_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        arc_caption.to_string(),
        font.clone(),
    ));

    let enlarge_caption = "Нарастить";
    let _enlarge_button = draw_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(enlarge_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        enlarge_caption.to_string(),
        font.clone(),
    ));

    let cut_caption = "Укоротить";
    let _cut_button = draw_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(cut_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        cut_caption.to_string(),
        font.clone(),
    ));

    draw_menu
}
