use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

pub fn create_options_menu(parent: &mut TabControl, font: &Font) -> Rc<RefCell<Container>> {
    let font_height = font.get_size("8").1 as i32 + 2;
    let menu_caption = "Опции";
    let options_menu = parent.add_tab(
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

    let font_size_caption = "Размер шрифта";
    let _font_size_button = options_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(font_size_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        font_size_caption.to_string(),
        font.clone(),
    ));

    let theme_caption = "Тёмная тема";
    let _theme_button = options_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(theme_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        theme_caption.to_string(),
        font.clone(),
    ));

    let smooth_caption = "Сглаживание линий";
    let _smooth_button = options_menu.borrow_mut().add_child(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(font.get_size(smooth_caption).0 as i32 + font_height),
            SizeConstraint::fixed(font_height),
        ),
        smooth_caption.to_string(),
        font.clone(),
    ));

    options_menu
}
