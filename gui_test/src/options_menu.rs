use std::cell::RefCell;
use std::rc::Rc;

use application::callback;
use application::callback_body;
use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;

use window::show_message;

use crate::config::*;
use crate::editor::*;
use crate::gui_helper::*;
use crate::GuiTest;
use crate::OPTIONS_MENU_INDEX;

pub fn create_options_menu(
    parent: &mut TabControl,
    font: &Font,
    editor: Rc<RefCell<Editor>>,
    context: Rc<RefCell<window::Context>>,
) -> Rc<RefCell<Container>> {
    let config = editor.borrow().config.clone();
    let font_height = font.get_size("8").1 as i32 + 2;
    let menu_caption = "Опции";
    let options_menu = parent.add_tab(
        menu_caption.to_string(),
        GuiSystem::default_size(&menu_caption, None, &font)
            .0
            .absolute,
        Container::new(
            SizeConstraints(SizeConstraint::flexible(0), SizeConstraint::fixed(0)),
            ContainerLayout::Vertical,
        ),
    );

    let font_size_line = options_menu.borrow_mut().add_child(Container::new(
        SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(font_height),
        ),
        ContainerLayout::Horizontal,
    ));

    let _font_size_caption = font_size_line
        .borrow_mut()
        .add_child(create_default_size_text_box("Размер шрифта:", font.clone()));
    let font_size_input = font_size_line
        .borrow_mut()
        .add_child(create_default_size_edit(
            "8888",
            font.clone(),
            context.borrow().clipboard.clone(),
        ));

    fn reset_font_input(config: Rc<RefCell<Config>>, font_size_input: Rc<RefCell<Edit>>) {
        font_size_input
            .borrow_mut()
            .set_text(&format!("{}", config.borrow().font_size.0));
    }

    reset_font_input(config.clone(), font_size_input.clone());

    font_size_input.borrow_mut().set_skip_callback(callback!(
        [font_size_input, config] () {
            reset_font_input(config.clone(), font_size_input.clone());
        }
    ));

    font_size_input.borrow_mut().set_enter_callback(callback!(
        [font_size_input, editor, context] (text) {
            let config = editor.borrow().config.clone();
            let old_font_size = config.borrow().font_size;
            match text.parse::<i32>() {
                Ok(new_font_size) => {
                    config.borrow_mut().font_size = ConfigFontSize(new_font_size).adjusted();
                    reset_font_input(config.clone(), font_size_input.clone());
                }
                Err(_) => {
                    show_message(
                        context.clone(),
                        &format!("{} - не число!", text),
                        "Ошибка ввода",
                    );
                    reset_font_input(config.clone(), font_size_input.clone());
                }
            }

            let new_font_size = config.borrow().font_size;
            if old_font_size != new_font_size {
                GuiTest::rebuild_gui(editor.clone(), context.clone(), OPTIONS_MENU_INDEX);
            }
        }
    ));

    let font_anti_aliasing_selector = options_menu.borrow_mut().add_child(RadioGroup::new(
        SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(font_height),
        ),
        ContainerLayout::Horizontal,
        Some(create_default_size_text_box(
            "Сглаживание шрифта:",
            font.clone(),
        )),
    ));

    let _no_font_anti_alisaing_button = font_anti_aliasing_selector
        .borrow_mut()
        .add_button(create_default_size_radio_button("Нету", font.clone()));

    let _font_anti_alisaing_button = font_anti_aliasing_selector
        .borrow_mut()
        .add_button(create_default_size_radio_button("Пиксельное", font.clone()));

    let _font_true_type_button =
        font_anti_aliasing_selector
            .borrow_mut()
            .add_button(create_default_size_radio_button(
                "Субпиксельное (true type)",
                font.clone(),
            ));

    font_anti_aliasing_selector
        .borrow_mut()
        .set_id(match config.borrow().font_aa_mode {
            FontAntiAliasingMode::NoAA => 0,
            FontAntiAliasingMode::AA => 1,
            FontAntiAliasingMode::TT => 2,
        });
    font_anti_aliasing_selector
        .borrow_mut()
        .set_callback(callback!([config, context] (aa_index) {
            let old_aa_index = config.borrow().font_aa_mode;
            match aa_index {
                0 => config.borrow_mut().font_aa_mode = FontAntiAliasingMode::NoAA,
                1 => config.borrow_mut().font_aa_mode = FontAntiAliasingMode::AA,
                2 => config.borrow_mut().font_aa_mode = FontAntiAliasingMode::TT,
                _ => {}
            }
            let new_aa_index = config.borrow().font_aa_mode;
            if new_aa_index != old_aa_index {
                    GuiTest::rebuild_gui(editor.clone(), context.clone(), OPTIONS_MENU_INDEX);

            }
        }));

    let line_anti_aliasing_selector = options_menu.borrow_mut().add_child(RadioGroup::new(
        SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(font_height),
        ),
        ContainerLayout::Horizontal,
        Some(create_default_size_text_box(
            "Сглаживание линий:",
            font.clone(),
        )),
    ));

    let _no_line_anti_aliasing_button = line_anti_aliasing_selector
        .borrow_mut()
        .add_button(create_default_size_radio_button("Нету", font.clone()));

    let _x2_line_anti_aliasing_button = line_anti_aliasing_selector
        .borrow_mut()
        .add_button(create_default_size_radio_button("Среднее", font.clone()));

    let _x4_line_anti_aliasing_button = line_anti_aliasing_selector
        .borrow_mut()
        .add_button(create_default_size_radio_button("Высшее", font.clone()));

    line_anti_aliasing_selector
        .borrow_mut()
        .set_id(match config.borrow().curves_aa_mode {
            CurvesAAMode::NoAntiAliasing => 0,
            CurvesAAMode::AntiAliasingX2 => 1,
            CurvesAAMode::AntiAliasingX4 => 2,
        });
    line_anti_aliasing_selector
        .borrow_mut()
        .set_callback(callback! ([config] (aa_index) {
            match aa_index {
                0 => config.borrow_mut().curves_aa_mode = CurvesAAMode::NoAntiAliasing,
                1 => config.borrow_mut().curves_aa_mode = CurvesAAMode::AntiAliasingX2,
                2 => config.borrow_mut().curves_aa_mode = CurvesAAMode::AntiAliasingX4,
                _ => {}
            }
        }));

    let theme_selector = options_menu.borrow_mut().add_child(RadioGroup::new(
        SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(font_height),
        ),
        ContainerLayout::Horizontal,
        Some(create_default_size_text_box("Цветовая тема:", font.clone())),
    ));

    let _dark_theme_button = theme_selector
        .borrow_mut()
        .add_button(create_default_size_radio_button("Тёмная", font.clone()));

    let _beige_theme_button = theme_selector
        .borrow_mut()
        .add_button(create_default_size_radio_button("Бежевая", font.clone()));

    let _light_theme_button = theme_selector
        .borrow_mut()
        .add_button(create_default_size_radio_button("Светлая", font.clone()));

    theme_selector
        .borrow_mut()
        .set_id(match config.borrow().color_theme {
            ColorTheme::Dark => 0,
            ColorTheme::Beige => 1,
            ColorTheme::Light => 2,
        });
    theme_selector
        .borrow_mut()
        .set_callback(callback!([config, context] (color_theme) {
                match color_theme {
                    0 => config.borrow_mut().color_theme = ColorTheme::Dark,
                    1 => config.borrow_mut().color_theme = ColorTheme::Beige,
                    2 => config.borrow_mut().color_theme = ColorTheme::Light,
                    _ => {}
                }

                context
                    .borrow_mut()
                    .gui_system
                    .set_color_theme(*get_gui_color_theme(&config.borrow()));
            }
        ));

    options_menu
}
