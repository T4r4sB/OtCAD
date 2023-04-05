use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

use crate::gui_helper::*;

pub fn create_group_menu(parent: &mut TabControl, font: &Font) -> Rc<RefCell<Container>> {
    let font_height = font.get_size("8").1 as i32 + 2;
    let menu_caption = "Группы";
    let group_menu = parent.add_tab(
        menu_caption.to_string(),
        GuiSystem::default_size(&menu_caption, None, &font)
            .0
            .absolute,
        Container::new(
            SizeConstraints(
                SizeConstraint::flexible(0),
                SizeConstraint::fixed(font_height),
            ),
            ContainerLayout::Horizontal,
        ),
    );

    let _group_button = group_menu
        .borrow_mut()
        .add_child(create_default_size_button("Объединить", font.clone()));

    let _ungroup_button = group_menu
        .borrow_mut()
        .add_child(create_default_size_button("Разрушить", font.clone()));

    let _add_button = group_menu
        .borrow_mut()
        .add_child(create_default_size_button("Добавить", font.clone()));

    let _rem_button = group_menu
        .borrow_mut()
        .add_child(create_default_size_button("Исключить", font.clone()));

    group_menu
}
