use std::cell::RefCell;
use std::rc::Rc;

use application::gui::*;

use crate::config::*;
use curves::curves::*;
use curves::points::*;
use curves::render::*;

pub struct CadColorTheme {
    line_color: u32,
    line_aa_color: u32,
}

static CAD_DARK_THEME: CadColorTheme = CadColorTheme {
    line_color: 0x88AA88,
    line_aa_color: 0xCCFFCC,
};

static CAD_BEIGE_THEME: CadColorTheme = CadColorTheme {
    line_color: 0x442200,
    line_aa_color: 0x442200,
};

static CAD_LIGHT_THEME: CadColorTheme = CadColorTheme {
    line_color: 0x000000,
    line_aa_color: 0x000000,
};

static BEIGE_THEME: GuiColorTheme = GuiColorTheme {
    background: 0xDDCCAA,
    font: 0x000000,
    splitter: 0x224466,
    highlight: 0x88AA66,
    pressed: 0x222222,
    selected: 0x4499CC,
    inactive: 0xAA9988,
    edit_focused: 0xEEEEEE,
};

pub fn get_cad_color_theme(config: &Config) -> &'static CadColorTheme {
    match config.color_theme {
        ColorTheme::Dark => &CAD_DARK_THEME,
        ColorTheme::Beige => &CAD_BEIGE_THEME,
        ColorTheme::Light => &CAD_LIGHT_THEME,
    }
}

pub fn get_gui_color_theme(config: &Config) -> &'static GuiColorTheme {
    match config.color_theme {
        ColorTheme::Dark => &DARK_THEME,
        ColorTheme::Beige => &BEIGE_THEME,
        ColorTheme::Light => &LIGHT_THEME,
    }
}

pub struct Editor {
    pub last_file_id: usize,
    pub config: Rc<RefCell<Config>>,
}

impl Editor {
    pub fn new(config: Config) -> Self {
        Self {
            last_file_id: 0,
            config: Rc::new(RefCell::new(config)),
        }
    }

    pub fn get_next_id(&mut self) -> usize {
        self.last_file_id += 1;
        self.last_file_id
    }
}

pub struct CadView {
    base: GuiControlBase,
    editor: Rc<RefCell<Editor>>,
}

impl std::fmt::Debug for CadView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base.fmt(f)
    }
}

impl CadView {
    pub fn new(size_constraints: SizeConstraints, editor: Rc<RefCell<Editor>>) -> Self {
        Self {
            base: GuiControlBase::new(size_constraints),
            editor,
        }
    }
}

impl GuiControl for CadView {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        match m {
            GuiMessage::Draw(buf, theme, force) => {
                if self.base.can_draw(force) {
                    GuiSystem::erase_background(buf, EmptySpaceState::Empty, theme);
                    let curve = Curve::<f32>::circle(Point::new(100.0, 100.0), 50.0);
                    let e = Entity::Curve(curve);
                    let mut buffer = vec![(0, 0); buf.get_size().1 * 4];
                    let cad_color_theme =
                        get_cad_color_theme(&self.editor.borrow().config.borrow());

                    match self.editor.borrow().config.borrow().curves_aa_mode {
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
            GuiMessage::MouseDown(_) => {
                return true;
            }
            GuiMessage::MouseUp(_, _) => {
                return true;
            }
            _ => return false,
        }
    }
}
