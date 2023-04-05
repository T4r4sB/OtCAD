use std::cell::RefCell;
use std::rc::Rc;

use application::callback;
use application::callback_body;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;
use application::keys::*;

use crate::editor::*;
use crate::gui_helper::*;

pub fn create_draw_menu(
    parent: &mut TabControl,
    font: &Font,
    editor: Rc<RefCell<Editor>>,
) -> Rc<RefCell<Container>> {
    let font_height = font.get_size("8").1 as i32 + 2;
    let menu_caption = "Рисовать";
    let draw_menu = parent.add_tab(
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

    let _line_button = draw_menu
        .borrow_mut()
        .add_child(create_default_size_button_with_hotkey(
            "Отрезок",
            font.clone(),
            Hotkey::new(Key::L),
            true,
        ));

    let _circle_button = draw_menu
        .borrow_mut()
        .add_child(create_default_size_button("Окружность", font.clone()));

    let _arc_button = draw_menu
        .borrow_mut()
        .add_child(create_default_size_button("Дуга", font.clone()));

    let _cut_enlarge_button = draw_menu.borrow_mut().add_child(create_default_size_button(
        "Нарастить/укоротить",
        font.clone(),
    ));

    let _skip_button = draw_menu.borrow_mut().add_child(
        create_default_size_button_with_hotkey(
            "Сброс",
            font.clone(),
            Hotkey::new(Key::Escape),
            true,
        )
        .callback(callback!([editor] () {
        editor.borrow_mut().skip_state()
        })),
    );

    draw_menu
}
