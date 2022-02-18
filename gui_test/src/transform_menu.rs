use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

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

    let translate_caption = "Сдвиг";
    let _translate_button = transform_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(translate_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        translate_caption.to_string(),
        font.clone(),
    ));

    let copy_caption = "Копия";
    let _copy_button = transform_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(copy_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        copy_caption.to_string(),
        font.clone(),
    ));

    let rotate_caption = "Поворот";
    let _rotate_button = transform_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(rotate_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        rotate_caption.to_string(),
        font.clone(),
    ));

    transform_menu
}
