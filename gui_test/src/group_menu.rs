use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

pub fn create_group_menu(parent: &mut TabControl, font: &Font) -> Rc<RefCell<Container>> {
    let font_height = font.get_size("8").1 as i32 + 2;
    let menu_caption = "Группы";
    let group_menu = parent.add_tab(
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

    let group_caption = "Объединить";
    let _group_button = group_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(group_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        group_caption.to_string(),
        font.clone(),
    ));

    let ungroup_caption = "Разрушить";
    let _ungroup_button = group_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(ungroup_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        ungroup_caption.to_string(),
        font.clone(),
    ));

    let add_caption = "Добавить";
    let _add_button = group_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(add_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        add_caption.to_string(),
        font.clone(),
    ));

    let rem_caption = "Исключить";
    let _rem_button = group_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(rem_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        rem_caption.to_string(),
        font.clone(),
    ));

    group_menu
}
