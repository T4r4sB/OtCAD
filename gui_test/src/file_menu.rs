use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

pub fn create_file_menu(parent: &mut TabControl, font: &Font) -> Rc<RefCell<Container>> {
    let font_height = font.get_size("8").1 as i32 + 2;
    let menu_caption = "Файл";
    let file_menu = parent.add_tab(
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

    let new_caption = "Новый";
    let _new_button = file_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(new_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        new_caption.to_string(),
        font.clone(),
    ));

    let open_caption = "Открыть";
    let _open_button = file_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(open_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        open_caption.to_string(),
        font.clone(),
    ));

    let save_caption = "Сохранить";
    let _save_button = file_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(save_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        save_caption.to_string(),
        font.clone(),
    ));

    let save_as_caption = "Сохранить как";
    let _save_as_button = file_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(save_as_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        save_as_caption.to_string(),
        font.clone(),
    ));

    file_menu
}
