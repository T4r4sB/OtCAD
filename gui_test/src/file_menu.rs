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

pub fn create_file_menu(
    parent: &mut TabControl,
    font: &Font,
    editor: Rc<RefCell<Editor>>,
) -> Rc<RefCell<Container>> {
    let menu_caption = "Файл";
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

    {
        let font = font.clone();
        let _new_button = default_panel.borrow_mut().add_child(
            create_default_size_button_with_hotkey(
                "Новый",
                font.clone(),
                Hotkey::ctrl(Key::N),
                true,
            )
            .callback(callback!([editor]() {
               new_file(font.clone(), editor);
            })),
        );
    }

    let _open_button =
        default_panel
            .borrow_mut()
            .add_child(create_default_size_button_with_hotkey(
                "Открыть",
                font.clone(),
                Hotkey::ctrl(Key::O),
                true,
            ));

    let _save_button =
        default_panel
            .borrow_mut()
            .add_child(create_default_size_button_with_hotkey(
                "Сохранить",
                font.clone(),
                Hotkey::ctrl(Key::S),
                true,
            ));

    let _save_as_button =
        default_panel
            .borrow_mut()
            .add_child(create_default_size_button_with_hotkey(
                "Сохранить как",
                font.clone(),
                Hotkey::ctrl_shift(Key::S),
                true,
            ));

    let close_button =
        default_panel
            .borrow_mut()
            .add_child(create_default_size_button_with_hotkey(
                "Закрыть",
                font.clone(),
                Hotkey::alt(Key::F3),
                true,
            ));

    close_button
        .borrow_mut()
        .set_callback(callback!([editor]() {
            editor.borrow_mut().close_selected_tab();
        }));

    let dxf_panel = file_menu.borrow_mut().add_child(Container::new(
        SizeConstraints(SizeConstraint::flexible(0), SizeConstraint::fixed(0)),
        ContainerLayout::Vertical,
    ));

    let _import_button = dxf_panel
        .borrow_mut()
        .add_child(create_default_size_button("Импорт из *.dxf", font.clone()));

    let _export_button = dxf_panel
        .borrow_mut()
        .add_child(create_default_size_button("Экспорт в *.dxf", font.clone()));

    let _export_many_button = dxf_panel.borrow_mut().add_child(create_default_size_button(
        "Экспорт контуров в разные *.dxf",
        font.clone(),
    ));

    file_menu
}

pub fn new_file(font: Font, editor: Rc<RefCell<Editor>>) {
    let document_id = editor.borrow_mut().add_random_document();
    editor
        .borrow_mut()
        .add_tab_by_existing_document(font, document_id, None);
}
