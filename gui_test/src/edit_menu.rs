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

pub fn create_edit_menu(
    parent: &mut TabControl,
    font: &Font,
    editor: Rc<RefCell<Editor>>,
) -> Rc<RefCell<Container>> {
    let menu_caption = "Правка";
    let file_menu = parent.add_tab(
        menu_caption.to_string(),
        GuiSystem::default_size(&menu_caption, None, &font)
            .0
            .absolute,
        Container::new(
            SizeConstraints(SizeConstraint::fixed(0), SizeConstraint::fixed(0)),
            ContainerLayout::Horizontal,
        ),
    );

    let default_panel = file_menu.borrow_mut().add_child(Container::new(
        SizeConstraints(SizeConstraint::fixed(0), SizeConstraint::fixed(0)),
        ContainerLayout::Vertical,
    ));

    let _delete_button = default_panel.borrow_mut().add_child(
        create_default_size_button_with_hotkey(
            "Удалить",
            font.clone(),
            Hotkey::new(Key::Delete),
            true,
        )
        .callback(callback!([editor](){
            editor.borrow_mut().remove_selected()
        })),
    );

    let _cut_button = default_panel
        .borrow_mut()
        .add_child(create_default_size_button_with_hotkey(
            "Вырезать",
            font.clone(),
            Hotkey::ctrl(Key::X),
            true,
        ));

    let _copy_button =
        default_panel
            .borrow_mut()
            .add_child(create_default_size_button_with_hotkey(
                "Копировать",
                font.clone(),
                Hotkey::ctrl(Key::C),
                true,
            ));

    let _paste_button =
        default_panel
            .borrow_mut()
            .add_child(create_default_size_button_with_hotkey(
                "Вставить",
                font.clone(),
                Hotkey::ctrl(Key::V),
                true,
            ));

    let time_machine_panel = file_menu.borrow_mut().add_child(Container::new(
        SizeConstraints(SizeConstraint::flexible(0), SizeConstraint::fixed(0)),
        ContainerLayout::Vertical,
    ));

    let _undo_dutton = time_machine_panel.borrow_mut().add_child(
        create_default_size_button_with_hotkey(
            "Отменить правку",
            font.clone(),
            Hotkey::ctrl(Key::Z),
            true,
        )
        .callback(callback!([editor](){
            editor.borrow_mut().undo()
        })),
    );

    let _redo_button = time_machine_panel.borrow_mut().add_child(
        create_default_size_button_with_hotkey(
            "Вернуть правку",
            font.clone(),
            Hotkey::ctrl(Key::Y),
            true,
        )
        .callback(callback!([editor](){
            editor.borrow_mut().redo()
        })),
    );

    file_menu
}
