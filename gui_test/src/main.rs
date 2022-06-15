use std::cell::RefCell;
use std::rc::Rc;

use application::draw_context::*;
use application::gui::gui_components::*;
use application::gui::*;

use window::*;

use bottom_panel::*;
use config::*;
use editor::*;
use file_menu::*;
use top_panel::*;

mod bottom_panel;
mod config;
mod draw_menu;
mod editor;
mod file_menu;
mod group_menu;
mod gui_helper;
mod options_menu;
mod top_panel;
mod transform_menu;

struct GuiTest {
    editor: Rc<RefCell<Editor>>,
}

impl GuiTest {
    fn new(config: Config) -> Self {
        Self {
            editor: Rc::new(RefCell::new(Editor::new(config))),
        }
    }

    fn rebuild_gui(
        editor: Rc<RefCell<Editor>>,
        context: Rc<RefCell<window::Context>>,
        top_panel_index: usize,
    ) {
        let config = editor.borrow().config.clone();
        let font_size = config.borrow().font_size.0;
        let font_aa_mode = config.borrow().font_aa_mode;

        let default_font =
            context
                .borrow_mut()
                .font_factory
                .new_font("MS Sans Serif", font_size, font_aa_mode);
        let font_height = default_font.get_size("8").1 as i32 + 2;

        let root = context.borrow_mut().gui_system.set_root(Container::new(
            SizeConstraints(SizeConstraint::flexible(0), SizeConstraint::flexible(0)),
            ContainerLayout::Vertical,
        ));

        let _hr = root
            .borrow_mut()
            .add_child(EmptySpace::new_splitter(SizeConstraints(
                SizeConstraint::flexible(0),
                SizeConstraint::fixed(1),
            )));

        let middle = root
            .borrow_mut()
            .add_child(TabControl::new(font_height, default_font.clone(), true).compressed());

        for _ in 1..7 {
            new_file(&default_font, editor.clone(), middle.clone());
        }

        create_top_panel(
            &mut root.borrow_mut(),
            &default_font,
            editor.clone(),
            middle.clone(),
            context.clone(),
            top_panel_index,
        );

        let _hr = root
            .borrow_mut()
            .add_child(EmptySpace::new_splitter(SizeConstraints(
                SizeConstraint::flexible(0),
                SizeConstraint::fixed(1),
            )));

        context
            .borrow_mut()
            .gui_system
            .set_color_theme(*get_gui_color_theme(&config.borrow()));
        create_bottom_panel(&mut root.borrow_mut(), &default_font, config.clone());
    }
}

impl window::Application for GuiTest {
    fn on_create(&mut self, context: Rc<RefCell<window::Context>>) {
        Self::rebuild_gui(self.editor.clone(), context.clone(), DRAW_MENU_INDEX);
    }

    fn on_close(&mut self, _context: Rc<RefCell<window::Context>>) {
        save_config(&self.editor.borrow().config.borrow());
    }

    fn on_change_position(&mut self, window_position: WindowPosition) {
        let config = self.editor.borrow_mut().config.clone();
        let config = &mut config.borrow_mut();
        let mut config_window_position = &mut config.window_position;
        if window_position.maximized {
            if let Some(config_window_position) = &mut config_window_position {
                config_window_position.maximized = true;
            } else {
                *config_window_position = Some(window_position);
            }
        } else {
            *config_window_position = Some(window_position);
        }
    }

    fn on_draw(&self, _: &mut DrawContext) {}
}

fn main() {
    let config = load_config().unwrap_or_default();
    let window_position = config.window_position;
    if let Err(_) =
        window::run_application("ОтКАД", Box::new(GuiTest::new(config)), window_position)
    {
        // Do nothing, read message and exit
    }
}
