use std::cell::RefCell;
use std::rc::Rc;

use application::draw_context::*;
use application::gui::gui_components::*;
use application::gui::*;

use curves::curves::*;
use curves::points::*;
use curves::render::*;

use window::*;

use bottom_panel::*;
use config::*;
use top_panel::*;

mod bottom_panel;
mod config;
mod draw_menu;
mod file_menu;
mod group_menu;
mod gui_helper;
mod options_menu;
mod top_panel;
mod transform_menu;

struct CadColorTheme {
    background_color: u32,
    line_color: u32,
    line_aa_color: u32,
}

static CAD_DARK_THEME: CadColorTheme = CadColorTheme {
    background_color: 0x000000,
    line_color: 0xAAAAAA,
    line_aa_color: 0xFFFFFF,
};

static CAD_BEIGE_THEME: CadColorTheme = CadColorTheme {
    background_color: 0xDDCCAA,
    line_color: 0x000000,
    line_aa_color: 0x000000,
};

static CAD_LIGHT_THEME: CadColorTheme = CadColorTheme {
    background_color: 0xFFFFFF,
    line_color: 0x000000,
    line_aa_color: 0x000000,
};

static BEIGE_THEME: GuiColorTheme = GuiColorTheme {
    font: 0x000000,
    splitter: 224466,
    highlight: 0x666666,
    pressed: 0x222222,
    selected: 0x4499CC,
    inactive: 0xAAAAAA,
    edit_highlight: 0xCCCCCC,
    edit_focused: 0xEEEEEE,
};

fn get_cad_color_theme(config: &Config) -> &'static CadColorTheme {
    match config.color_theme {
        ColorTheme::Dark => &CAD_DARK_THEME,
        ColorTheme::Beige => &CAD_BEIGE_THEME,
        ColorTheme::Light => &CAD_LIGHT_THEME,
    }
}

fn get_gui_color_theme(config: &Config) -> &'static GuiColorTheme {
    match config.color_theme {
        ColorTheme::Dark => &DARK_THEME,
        ColorTheme::Beige => &BEIGE_THEME,
        ColorTheme::Light => &LIGHT_THEME,
    }
}

#[derive(Debug)]
pub struct CadView {
    base: GuiControlBase,
    config: Rc<RefCell<Config>>,
}

impl CadView {
    pub fn new(size_constraints: SizeConstraints, config: Rc<RefCell<Config>>) -> Self {
        Self {
            base: GuiControlBase::new(size_constraints),
            config,
        }
    }
}

impl GuiControl for CadView {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        match m {
            GuiMessage::Draw(buf, _) => {
                if self.base.visible {
                    let curve = Curve::<f32>::circle(Point::new(100.0, 100.0), 50.0);
                    let e = Entity::Curve(curve);
                    let mut buffer = vec![(0, 0); buf.get_size().1 * 4];
                    let cad_color_theme = get_cad_color_theme(&self.config.borrow());

                    match self.config.borrow().curves_aa_mode {
                        CurvesAAMode::NoAntiAliasing => {
                            draw_curve(buf, &e, cad_color_theme.line_color, 1.0, &mut buffer, 1)
                        }
                        CurvesAAMode::AntiAliasingX2 => {
                            draw_curve(buf, &e, cad_color_theme.line_aa_color, 1.0, &mut buffer, 2)
                        }
                        CurvesAAMode::AntiAliasingX4 => {
                            draw_curve(buf, &e, cad_color_theme.line_aa_color, 1.0, &mut buffer, 4)
                        }
                    };
                }

                return true;
            }
            _ => return false,
        }
    }
}

struct GuiTest {
    config: Rc<RefCell<Config>>,
}

impl GuiTest {
    fn new(config: Rc<RefCell<Config>>) -> Self {
        Self { config }
    }

    fn rebuild_gui(
        config: Rc<RefCell<Config>>,
        context: Rc<RefCell<window::Context>>,
        top_panel_index: usize,
    ) {
        let font_size = match config.borrow().font_size {
            FontSize::Small => 15,
            FontSize::Average => 20,
            FontSize::Big => 31,
        };

        let font_aa_mode = config.borrow().font_aa_mode;

        let default_font =
            context
                .borrow_mut()
                .font_factory
                .new_font("MS Sans Serif", font_size, font_aa_mode);
        let root = context.borrow_mut().gui_system.set_root(Container::new(
            SizeConstraints(SizeConstraint::flexible(0), SizeConstraint::flexible(0)),
            ContainerLayout::Vertical,
        ));

        create_top_panel(
            &mut root.borrow_mut(),
            &default_font,
            config.clone(),
            context.clone(),
            top_panel_index,
        );
        let _hr = root.borrow_mut().add_child(ColorBox::new(SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(1),
        )));

        let _middle = root.borrow_mut().add_child(CadView::new(
            SizeConstraints(SizeConstraint::flexible(100), SizeConstraint::flexible(100)),
            config.clone(),
        ));

        let _hr = root.borrow_mut().add_child(ColorBox::new(SizeConstraints(
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
        Self::rebuild_gui(self.config.clone(), context.clone(), DRAW_MENU_INDEX);
    }

    fn on_close(&mut self, _context: Rc<RefCell<window::Context>>) {
        save_config(&self.config.borrow());
    }

    fn on_change_position(&mut self, window_position: WindowPosition) {
        if window_position.maximized {
            if let Some(window_position) = &mut self.config.borrow_mut().window_position {
                window_position.maximized = true;
            } else {
                self.config.borrow_mut().window_position = Some(window_position);
            }
        } else {
            self.config.borrow_mut().window_position = Some(window_position);
        }
    }

    fn on_draw(&self, draw_context: &mut DrawContext) {
        draw_context.buffer.fill(|p| {
            *p = get_cad_color_theme(&self.config.borrow()).background_color;
        });
    }
}

fn main() {
    let config = load_config().unwrap_or_default();
    let window_position = config.window_position;
    if let Err(_) = window::run_application(
        "ОтКАД",
        Box::new(GuiTest::new(Rc::new(RefCell::new(config)))),
        window_position,
    ) {
        // Do nothing, read message and exit
    }
}
