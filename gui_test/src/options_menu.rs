use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

pub fn create_options_menu(parent: &mut TabControl, font: &Font) -> Rc<RefCell<Container>> {
    let font_height = font.get_size("8").1 as i32 + 2;
    let check_width = font.get_size("V").0 as i32;
    let menu_caption = "Опции";
    let options_menu = parent.add_tab(
        menu_caption.to_string(),
        font.get_size(menu_caption).0 as i32 + font_height,
        Container::new(
            SizeConstraints(SizeConstraint::flexible(0), SizeConstraint::fixed(0)),
            ContainerLayout::Vertical,
        ),
    );

    let font_size_caption = " Размер шрифта:";
    let font_size_selector = options_menu.borrow_mut().add_child(RadioGroup::new(
        SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(font_height),
        ),
        ContainerLayout::Horizontal,
        TextBox::new(
            SizeConstraints(
                SizeConstraint::fixed(font.get_size(font_size_caption).0 as i32),
                SizeConstraint::fixed(font_height),
            ),
            font_size_caption.to_string(),
            font.clone(),
        ),
    ));

    let small_font_caption = "Мелкий";
    let _small_font_button = font_size_selector.borrow_mut().add_button(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(
                font.get_size(small_font_caption).0 as i32 + check_width + font_height,
            ),
            SizeConstraint::fixed(font_height),
        ),
        small_font_caption.to_string(),
        font.clone(),
    ));

    let average_font_caption = "Средний";
    let _average_font_button = font_size_selector.borrow_mut().add_button(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(
                font.get_size(average_font_caption).0 as i32 + check_width + font_height,
            ),
            SizeConstraint::fixed(font_height),
        ),
        average_font_caption.to_string(),
        font.clone(),
    ));

    let big_font_caption = "Большой";
    let __font_button = font_size_selector.borrow_mut().add_button(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(
                font.get_size(big_font_caption).0 as i32 + check_width + font_height,
            ),
            SizeConstraint::fixed(font_height),
        ),
        big_font_caption.to_string(),
        font.clone(),
    ));

    let line_aliasing_caption = " Сглаживание линий:";
    let line_aliasing_selector = options_menu.borrow_mut().add_child(RadioGroup::new(
        SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(font_height),
        ),
        ContainerLayout::Horizontal,
        TextBox::new(
            SizeConstraints(
                SizeConstraint::fixed(font.get_size(line_aliasing_caption).0 as i32),
                SizeConstraint::fixed(font_height),
            ),
            line_aliasing_caption.to_string(),
            font.clone(),
        ),
    ));

    let no_line_alisaing_caption = "Нету";
    let _no_line_alisaing_button = line_aliasing_selector.borrow_mut().add_button(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(
                font.get_size(no_line_alisaing_caption).0 as i32 + check_width + font_height,
            ),
            SizeConstraint::fixed(font_height),
        ),
        no_line_alisaing_caption.to_string(),
        font.clone(),
    ));

    let x2_line_alisaing_caption = "Среднее";
    let _x2_line_alisaing_button = line_aliasing_selector.borrow_mut().add_button(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(
                font.get_size(x2_line_alisaing_caption).0 as i32 + check_width + font_height,
            ),
            SizeConstraint::fixed(font_height),
        ),
        x2_line_alisaing_caption.to_string(),
        font.clone(),
    ));

    let x4_line_alisaing_caption = "Высшее";
    let _x4_line_alisaing_button = line_aliasing_selector.borrow_mut().add_button(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(
                font.get_size(x4_line_alisaing_caption).0 as i32 + check_width + font_height,
            ),
            SizeConstraint::fixed(font_height),
        ),
        x4_line_alisaing_caption.to_string(),
        font.clone(),
    ));

    let theme_caption = " Цветовая тема:";
    let theme_selector = options_menu.borrow_mut().add_child(RadioGroup::new(
        SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(font_height),
        ),
        ContainerLayout::Horizontal,
        TextBox::new(
            SizeConstraints(
                SizeConstraint::fixed(font.get_size(theme_caption).0 as i32),
                SizeConstraint::fixed(font_height),
            ),
            theme_caption.to_string(),
            font.clone(),
        ),
    ));

    let dark_theme_caption = "Тёмная";
    let _dark_theme_button = theme_selector.borrow_mut().add_button(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(
                font.get_size(dark_theme_caption).0 as i32 + check_width + font_height,
            ),
            SizeConstraint::fixed(font_height),
        ),
        dark_theme_caption.to_string(),
        font.clone(),
    ));

    let beige_theme_caption = "Бежевая";
    let _beige_theme_button = theme_selector.borrow_mut().add_button(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(
                font.get_size(beige_theme_caption).0 as i32 + check_width + font_height,
            ),
            SizeConstraint::fixed(font_height),
        ),
        beige_theme_caption.to_string(),
        font.clone(),
    ));

    let light_theme_caption = "Светлая";
    let _light_theme_button = theme_selector.borrow_mut().add_button(Button::new(
        SizeConstraints(
            SizeConstraint::fixed(
                font.get_size(light_theme_caption).0 as i32 + check_width + font_height,
            ),
            SizeConstraint::fixed(font_height),
        ),
        light_theme_caption.to_string(),
        font.clone(),
    ));

    options_menu
}
