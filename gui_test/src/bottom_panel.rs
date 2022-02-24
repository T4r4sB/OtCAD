use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

use crate::config::*;
use crate::gui_helper::*;

pub fn create_bottom_panel(
    root: &mut Container,
    font: &Font,
    config: Rc<RefCell<Config>>,
) -> Rc<RefCell<Container>> {
    let font_symbol_size = font.get_size("8");
    let font_height = font_symbol_size.1 as i32 + 2;

    let bottom_panel = root.add_child(Container::new(
        SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(font_height),
        ),
        ContainerLayout::Horizontal,
    ));

    let config_capture = Rc::downgrade(&config);
    let _grid_button = bottom_panel.borrow_mut().add_child(
        create_default_size_check_button("Показать сетку", font.clone())
            .check_box(config.borrow().show_grid)
            .checkbox_callback(move |c| {
                config_capture.upgrade().map(|config| {
                    config.borrow_mut().show_grid = c;
                });
            }),
    );

    let _hr = bottom_panel
        .borrow_mut()
        .add_child(ColorBox::new(SizeConstraints(
            SizeConstraint::fixed(1),
            SizeConstraint::flexible(0),
        )));

    let _snap_button = bottom_panel
        .borrow_mut()
        .add_child(create_default_size_text_box("Привязки:", font.clone()));

    let config_capture = Rc::downgrade(&config);
    let _grid_nodes_button = bottom_panel.borrow_mut().add_child(
        create_default_size_check_button("Узлы сетки", font.clone())
            .check_box(config.borrow().snap_options.snap_grid)
            .checkbox_callback(move |c| {
                config_capture.upgrade().map(|config| {
                    config.borrow_mut().snap_options.snap_grid = c;
                });
            }),
    );

    let _es = bottom_panel
        .borrow_mut()
        .add_child(EmptySpace::new(SizeConstraints(
            SizeConstraint::fixed(font_symbol_size.0 as i32 / 2),
            SizeConstraint::flexible(0),
        )));

    let config_capture = Rc::downgrade(&config);
    let _endpoints_button = bottom_panel.borrow_mut().add_child(
        create_default_size_check_button("Концы", font.clone())
            .check_box(config.borrow().snap_options.snap_endpoints)
            .checkbox_callback(move |c| {
                config_capture.upgrade().map(|config| {
                    config.borrow_mut().snap_options.snap_endpoints = c;
                });
            }),
    );

    let _es = bottom_panel
        .borrow_mut()
        .add_child(EmptySpace::new(SizeConstraints(
            SizeConstraint::fixed(font_symbol_size.0 as i32 / 2),
            SizeConstraint::flexible(0),
        )));

    let config_capture = Rc::downgrade(&config);
    let _intersections_button = bottom_panel.borrow_mut().add_child(
        create_default_size_check_button("Пересечения", font.clone())
            .check_box(config.borrow().snap_options.snap_crosses)
            .checkbox_callback(move |c| {
                config_capture.upgrade().map(|config| {
                    config.borrow_mut().snap_options.snap_crosses = c;
                });
            }),
    );

    let _es = bottom_panel
        .borrow_mut()
        .add_child(EmptySpace::new(SizeConstraints(
            SizeConstraint::fixed(font_symbol_size.0 as i32 / 2),
            SizeConstraint::flexible(0),
        )));

    let config_capture = Rc::downgrade(&config);
    let _centers_button = bottom_panel.borrow_mut().add_child(
        create_default_size_check_button("Центры дуг", font.clone())
            .check_box(config.borrow().snap_options.snap_centers)
            .checkbox_callback(move |c| {
                config_capture.upgrade().map(|config| {
                    config.borrow_mut().snap_options.snap_centers = c;
                });
            }),
    );

    bottom_panel
}
