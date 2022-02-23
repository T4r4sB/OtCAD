use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

use crate::gui_helper::*;

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

    let _line_button = draw_menu
        .borrow_mut()
        .add_child(create_default_size_button("Отрезок", font.clone()));

    let _circle_button = draw_menu
        .borrow_mut()
        .add_child(create_default_size_button("Круг", font.clone()));

    let _arc_button = draw_menu
        .borrow_mut()
        .add_child(create_default_size_button("Дуга", font.clone()));

    let _enlarge_button = draw_menu
        .borrow_mut()
        .add_child(create_default_size_button("Нарастить", font.clone()));

    let _cut_button = draw_menu
        .borrow_mut()
        .add_child(create_default_size_button("Укоротить", font.clone()));

    draw_menu
}
