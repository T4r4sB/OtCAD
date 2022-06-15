use std::cell::RefCell;
use std::rc::Rc;

use application::font::*;
use application::gui::gui_components::*;

use crate::draw_menu::*;
use crate::editor::*;
use crate::file_menu::*;
use crate::group_menu::*;
use crate::options_menu::*;
use crate::transform_menu::*;

pub static DRAW_MENU_INDEX: usize = 1;
pub static OPTIONS_MENU_INDEX: usize = 4;

pub fn create_top_panel(
    root: &mut Container,
    font: &Font,
    editor: Rc<RefCell<Editor>>,
    draws: Rc<RefCell<TabControl>>,
    context: Rc<RefCell<window::Context>>,
    top_panel_index: usize,
) -> Rc<RefCell<TabControl>> {
    let font_height = font.get_size("8").1 as i32 + 2;
    let top_panel = root.insert_child(0, TabControl::new(font_height, font.clone(), false));

    create_file_menu(&mut top_panel.borrow_mut(), font, editor.clone(), draws);
    create_draw_menu(&mut top_panel.borrow_mut(), font);
    create_group_menu(&mut top_panel.borrow_mut(), font);
    create_transform_menu(&mut top_panel.borrow_mut(), font);
    create_options_menu(&mut top_panel.borrow_mut(), font, editor, context);
    top_panel.borrow_mut().select_tab(top_panel_index);

    top_panel
}
