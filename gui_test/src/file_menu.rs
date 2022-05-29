use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

use crate::gui_helper::*;

pub fn create_file_menu(parent: &mut TabControl, font: &Font) -> Rc<RefCell<Container>> {
    let font_height = font.get_size("8").1 as i32 + 2;
    let menu_caption = "Файл";
    let file_menu = parent.add_tab(
        menu_caption.to_string(),
        font.get_size(menu_caption).0 as i32 + font_height,
        Container::new(
            SizeConstraints(SizeConstraint::flexible(0), SizeConstraint::fixed(0)),
            ContainerLayout::Vertical,
        ),
    );

    let default_panel = file_menu.borrow_mut().add_child(Container::new(
        SizeConstraints(SizeConstraint::flexible(0), SizeConstraint::fixed(0)),
        ContainerLayout::Horizontal,
    ));

    let _new_button = default_panel.borrow_mut().add_child(
        create_default_size_button("Новый", font.clone()).callback(|| {
            panic!("test panic");
        }),
    );

    let _open_button = default_panel
        .borrow_mut()
        .add_child(create_default_size_button("Открыть", font.clone()));

    let _save_button = default_panel
        .borrow_mut()
        .add_child(create_default_size_button("Сохранить", font.clone()));

    let _save_as_button = default_panel
        .borrow_mut()
        .add_child(create_default_size_button("Сохранить как", font.clone()));

    let _close_button = default_panel
        .borrow_mut()
        .add_child(create_default_size_button("Закрыть", font.clone()));

    let _es = file_menu
        .borrow_mut()
        .add_child(EmptySpace::new(SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(font_height / 4),
        )));

    let dxf_panel = file_menu.borrow_mut().add_child(Container::new(
        SizeConstraints(SizeConstraint::flexible(0), SizeConstraint::fixed(0)),
        ContainerLayout::Horizontal,
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
