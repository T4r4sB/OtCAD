use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

use crate::gui_helper::*;

pub fn create_transform_menu(parent: &mut TabControl, font: &Font) -> Rc<RefCell<Container>> {
    let font_height = font.get_size("8").1 as i32 + 2;

    let menu_caption = "Преобразовать";
    let transform_menu = parent.add_tab(
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

    let _translate_button = transform_menu
        .borrow_mut()
        .add_child(create_default_size_button("Сдвиг", font.clone()));

    let _copy_button = transform_menu
        .borrow_mut()
        .add_child(create_default_size_button("Копия", font.clone()));

    let _rotate_button = transform_menu
        .borrow_mut()
        .add_child(create_default_size_button("Поворот", font.clone()));

    let _rotate_array_button = transform_menu
        .borrow_mut()
        .add_child(create_default_size_button("Круговой массив", font.clone()));

    transform_menu
}
