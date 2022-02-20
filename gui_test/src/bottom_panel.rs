use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

pub fn create_bottom_panel(root: &mut Container, font: &Font) -> Rc<RefCell<Container>> {
    let font_height = font.get_size("8").1 as i32 + 2;
    let check_width = font.get_size("V").0 as i32;

    let bottom_panel = root.add_child(Container::new(
        SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(font_height),
        ),
        ContainerLayout::Horizontal,
    ));

    let grid_caption = "Сетка";
    let _grid_button = bottom_panel.borrow_mut().add_child(
        Button::new(
            SizeConstraints(
                SizeConstraint::fixed(
                    font.get_size(grid_caption).0 as i32 + check_width + font_height,
                ),
                SizeConstraint::fixed(font_height),
            ),
            grid_caption.to_string(),
            font.clone(),
        )
        .check_box(),
    );

    let _hr = bottom_panel
        .borrow_mut()
        .add_child(ColorBox::new(SizeConstraints(
            SizeConstraint::fixed(1),
            SizeConstraint::fixed(font_height),
        )));

    let snap_caption = " Привязки:";
    let _snap_button = bottom_panel.borrow_mut().add_child(TextBox::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(snap_caption).0 as i32),
            SizeConstraint::fixed(font_height),
        ),
        snap_caption.to_string(),
        font.clone(),
    ));

    let grid_nodes_caption = "Узлы сетки";
    let _grid_nodes_button = bottom_panel.borrow_mut().add_child(
        Button::new(
            SizeConstraints(
                SizeConstraint::fixed(
                    font.get_size(grid_nodes_caption).0 as i32 + check_width + font_height,
                ),
                SizeConstraint::fixed(font_height),
            ),
            grid_nodes_caption.to_string(),
            font.clone(),
        )
        .check_box(),
    );

    let endpoints_caption = "Концы";
    let _endpoints_button = bottom_panel.borrow_mut().add_child(
        Button::new(
            SizeConstraints(
                SizeConstraint::fixed(
                    font.get_size(endpoints_caption).0 as i32 + check_width + font_height,
                ),
                SizeConstraint::fixed(font_height),
            ),
            endpoints_caption.to_string(),
            font.clone(),
        )
        .check_box(),
    );

    let intersections_caption = "Пересечения";
    let _intersections_button = bottom_panel.borrow_mut().add_child(
        Button::new(
            SizeConstraints(
                SizeConstraint::fixed(
                    font.get_size(intersections_caption).0 as i32 + check_width + font_height,
                ),
                SizeConstraint::fixed(font_height),
            ),
            intersections_caption.to_string(),
            font.clone(),
        )
        .check_box(),
    );

    bottom_panel
}
