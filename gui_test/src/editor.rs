use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use application::callback;
use application::callback_body;
use application::font::*;
use application::gui::gui_components::*;
use application::gui::*;
use application::image::*;
use application::keys::*;

use crate::config::*;
use crate::document::*;
use crate::picts::*;
use locc::points::*;
use locc::render::*;
use locc::*;

use rand::*;

pub struct CadColorTheme {
    line_color: u32,
    line_aa_color: u32,
    highlight_line_color: u32,
    highlight_line_aa_color: u32,
    grid_color_base: u32,
    grid_color_sub: u32,
    grid_font: u32,
    selection_rect_color: u32,
    selection_bevel_color: u32,
    pic_color: u32,
}

static CAD_DARK_THEME: CadColorTheme = CadColorTheme {
    line_color: 0x88AA88,
    line_aa_color: 0xCCFFCC,
    highlight_line_color: 0xAA8800,
    highlight_line_aa_color: 0xFFCC00,
    grid_color_base: 0x444444,
    grid_color_sub: 0x282828,
    grid_font: 0x808080,
    selection_rect_color: 0x3F2F00,
    selection_bevel_color: 0xBF8F00,
    pic_color: 0xBF8F00,
};

static CAD_BEIGE_THEME: CadColorTheme = CadColorTheme {
    line_color: 0x002244,
    line_aa_color: 0x002244,
    highlight_line_color: 0x0088FF,
    highlight_line_aa_color: 0x0088FF,
    grid_color_base: 0xA09070,
    grid_color_sub: 0xC0B090,
    grid_font: 0x908060,
    selection_rect_color: 0x001F3F,
    selection_bevel_color: 0x007FFF,
    pic_color: 0x007FFF,
};

static CAD_LIGHT_THEME: CadColorTheme = CadColorTheme {
    line_color: 0x000000,
    line_aa_color: 0x000000,
    highlight_line_color: 0x338800,
    highlight_line_aa_color: 0x338800,
    grid_color_base: 0xBBBBBB,
    grid_color_sub: 0xDDDDDD,
    grid_font: 0xAAAAAA,
    selection_rect_color: 0x1F2F0F,
    selection_bevel_color: 0x3F5F00,
    pic_color: 0x3F8000,
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

struct EditorInternal {
    pub selected_document_id: usize,
    pub documents: HashMap<usize, Rc<RefCell<Document>>>,
    pub tab_id_to_document_id: HashMap<usize, usize>,
}
pub struct Editor {
    pub last_document_id: usize,
    pub config: Rc<RefCell<Config>>,
    pub tab_control: Option<Rc<RefCell<TabControl>>>,
    pub picts: Rc<RefCell<Picts>>,
    internal: Rc<RefCell<EditorInternal>>,
}

impl Editor {
    pub fn new(config: Config) -> Self {
        Self {
            last_document_id: 0,
            config: Rc::new(RefCell::new(config)),
            tab_control: None,
            picts: Rc::new(RefCell::new(Picts::new())),
            internal: Rc::new(RefCell::new(EditorInternal {
                selected_document_id: 1,
                documents: HashMap::new(),
                tab_id_to_document_id: HashMap::new(),
            })),
        }
    }

    pub fn add_tab_by_existing_document(
        &self,
        font: Font,
        document_id: usize,
        force_selected_document_id: Option<usize>,
    ) {
        let document =
            if let Some(document) = self.internal.borrow_mut().documents.get(&document_id) {
                document.clone()
            } else {
                return;
            };
        let tab_control = if let Some(tab_control) = &self.tab_control {
            tab_control.clone()
        } else {
            return;
        };
        let new_file_caption = format!("Новый чертёж {}", document_id);

        let font_height = font.get_size("8").1 as i32 + 2;
        let mut tab_content = Container::new(
            SizeConstraints(SizeConstraint::flexible(0), SizeConstraint::flexible(0)),
            ContainerLayout::Vertical,
        );

        tab_content.add_child(TextBox::new(
            SizeConstraints(
                SizeConstraint::flexible(0),
                SizeConstraint::fixed(font_height),
            ),
            new_file_caption.clone(),
            font.clone(),
        ));

        tab_content.add_child(EmptySpace::new_splitter(SizeConstraints(
            SizeConstraint::flexible(0),
            SizeConstraint::fixed(1),
        )));

        tab_content.add_child(CadView::new(
            SizeConstraints(SizeConstraint::flexible(200), SizeConstraint::flexible(200)),
            document.clone(),
            self.picts.clone(),
            self.config.clone(),
            font.clone(),
        ));

        let (_cad_tab, tab_id) = tab_control.borrow_mut().add_tab_with_id(
            new_file_caption.clone(),
            GuiSystem::default_size(&new_file_caption, None, &font)
                .0
                .absolute,
            tab_content,
        );

        self.internal
            .borrow_mut()
            .tab_id_to_document_id
            .insert(tab_id, document_id);

        if force_selected_document_id
            .map(|f| f == document_id)
            .unwrap_or(true)
        {
            tab_control.borrow_mut().select_tab(tab_id);
        }
    }

    fn close_tab_impl(
        internal: Rc<RefCell<EditorInternal>>,
        tab_control: Rc<RefCell<TabControl>>,
        id: usize,
    ) {
        tab_control.borrow_mut().delete_tab(id);
        let mut internal = internal.borrow_mut();
        if let Some(document_id) = internal.tab_id_to_document_id.get(&id).copied() {
            internal.documents.remove(&document_id);
        }
        internal.tab_id_to_document_id.remove(&id);
    }

    pub fn close_selected_tab(&mut self) {
        if let Some(tab_control) = &self.tab_control {
            Self::close_tab_impl(
                self.internal.clone(),
                tab_control.clone(),
                tab_control.borrow().selected_tab_id(),
            );
        }
    }

    pub fn set_tab_control(&mut self, font: Font, tab_control: Rc<RefCell<TabControl>>) {
        let internal = self.internal.clone();

        tab_control
            .borrow_mut()
            .set_change_tab_callback(callback!( [internal] (tab_id) {
                let mut internal = internal.borrow_mut();
                if let Some(document_id) = internal.tab_id_to_document_id.get(&tab_id) {
                    internal.selected_document_id = *document_id;
                }
            }));

        tab_control
            .borrow_mut()
            .set_close_tab_callback(callback!( [tab_control, internal] (id) {
                Self::close_tab_impl(internal, tab_control, id);
            }));

        self.tab_control = Some(tab_control);

        let mut ids: Vec<_> = internal.borrow_mut().documents.keys().copied().collect();
        ids.sort();
        let selected_document_id = internal.borrow_mut().selected_document_id;
        for id in ids {
            self.add_tab_by_existing_document(font.clone(), id, Some(selected_document_id));
        }
    }

    fn get_next_id(&mut self) -> usize {
        self.last_document_id += 1;
        self.last_document_id
    }

    pub fn add_random_document(&mut self) -> usize {
        let mut document = Document::new();
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            let c = CLoCC::circle(
                Point::new(
                    rng.gen_range::<f64, _>(0.0..10.0),
                    rng.gen_range::<f64, _>(0.0..10.0),
                ),
                rng.gen_range(1.0..2.0),
            );
            let new_entity = crate::document::LoCC::new_clocc(c);
            document.add_entity(new_entity);
        }

        for _ in 0..10 {
            let s = SoCC::line(
                Point::new(
                    rng.gen_range::<f64, _>(0.0..10.0),
                    rng.gen_range::<f64, _>(0.0..10.0),
                ),
                Point::new(
                    rng.gen_range::<f64, _>(0.0..10.0),
                    rng.gen_range::<f64, _>(0.0..10.0),
                ),
            );
            let new_entity = crate::document::LoCC::new_socc(s);
            document.add_entity(new_entity);
        }

        let c = CLoCC::circle(Point::new(0.0, 0.0), 10.0);
        document.add_entity(crate::document::LoCC::new_clocc(c));
        let c = CLoCC::circle(Point::new(15.0, 5.0), 5.0);
        document.add_entity(crate::document::LoCC::new_clocc(c));
        document.fix_history();

        let document_id = self.get_next_id();
        self.internal
            .borrow_mut()
            .documents
            .insert(document_id, Rc::new(RefCell::new(document)));
        document_id
    }

    pub fn get_active_document(&self) -> Option<Rc<RefCell<Document>>> {
        let internal = self.internal.borrow();
        internal
            .documents
            .get(&internal.selected_document_id)
            .cloned()
    }

    pub fn skip_state(&self) {
        if let Some(document) = self.get_active_document() {
            document.borrow_mut().skip_state();
        }
    }

    pub fn remove_selected(&self) {
        if let Some(document) = self.get_active_document() {
            document.borrow_mut().remove_selected();
        }
    }

    pub fn undo(&self) {
        if let Some(document) = self.get_active_document() {
            document.borrow_mut().undo();
        }
    }

    pub fn redo(&self) {
        if let Some(document) = self.get_active_document() {
            document.borrow_mut().redo();
        }
    }
}

pub struct CadView {
    base: GuiControlBase,
    document: Rc<RefCell<Document>>,
    picts: Rc<RefCell<Picts>>,
    config: Rc<RefCell<Config>>,
    font: Font,
}

impl std::fmt::Debug for CadView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base.fmt(f)
    }
}

impl CadView {
    pub fn new(
        size_constraints: SizeConstraints,
        document: Rc<RefCell<Document>>,
        picts: Rc<RefCell<Picts>>,
        config: Rc<RefCell<Config>>,
        font: Font,
    ) -> Self {
        Self {
            base: GuiControlBase::new(size_constraints),
            document,
            picts,
            config,
            font,
        }
    }

    pub fn screen_coord_to_document_coord(
        &mut self,
        position: Point<f64>,
    ) -> (Point<f64>, Point<f64>) {
        let rect = self.get_base_mut().get_rect();
        let screen_center = Point::new(
            (rect.right_bottom.0 as f64 + rect.left_top.0 as f64) * 0.5,
            (rect.right_bottom.1 as f64) * 0.5,
        );
        let document = self.document.borrow();
        let rel_position = position - screen_center;
        (
            rel_position,
            rel_position.scale(1.0 / document.get_scale()) + document.get_center(),
        )
    }
}

impl GuiControl for CadView {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        match m {
            GuiMessage::GetHotkeys(hotkey_map, active) => {
                if active {
                    let document = self.document.clone();
                    let mut add_shifting_key = |hotkey: Hotkey, shift: Point<f64>| {
                        hotkey_map.insert(hotkey, HotkeyCallback::new(Rc::new(
                        callback!([document]() {
                            let mut document = document.borrow_mut();
                            let new_center = document.get_center() + shift.scale(1.0 / document.get_scale());
                            document.set_center(new_center);
                        }
                        ))));
                    };

                    add_shifting_key(Hotkey::shift(Key::Right), Point::new(1.0, 0.0));
                    add_shifting_key(Hotkey::new(Key::Right), Point::new(10.0, 0.0));
                    add_shifting_key(Hotkey::shift(Key::Down), Point::new(0.0, 1.0));
                    add_shifting_key(Hotkey::new(Key::Down), Point::new(0.0, 10.0));
                    add_shifting_key(Hotkey::shift(Key::Left), Point::new(-1.0, 0.0));
                    add_shifting_key(Hotkey::new(Key::Left), Point::new(-10.0, 0.0));
                    add_shifting_key(Hotkey::shift(Key::Up), Point::new(0.0, -1.0));
                    add_shifting_key(Hotkey::new(Key::Up), Point::new(0.0, -10.0));
                }
                return true;
            }
            GuiMessage::MouseWheel(position, delta) => {
                let (rel_position, document_position) = self.screen_coord_to_document_coord(
                    Point::new(position.0 as f64, position.1 as f64),
                );
                let mut document = self.document.borrow_mut();
                document.change_scale(-delta * 10);
                let new_center = document_position - rel_position.scale(1.0 / document.get_scale());
                document.set_center(new_center);
                return true;
            }
            GuiMessage::Draw(buf, theme, force) => {
                if self.base.can_draw(force) {
                    GuiSystem::erase_background(buf, EmptySpaceState::Empty, theme);
                    let document = self.document.borrow();
                    let config = self.config.borrow();
                    let scale = document.get_scale();
                    let center = document.get_center();
                    let buf_f64 = Point::new(buf.get_size().0 as f64, buf.get_size().1 as f64);
                    let buf_center = (buf_f64
                        - Point::new(0.0, self.base.get_rect().left_top.1 as f64))
                    .scale(0.5);
                    let cad_color_theme = get_cad_color_theme(&config);
                    if config.show_grid {
                        let grid_step = document.get_grid_step();
                        fn each_grid_line(
                            step: f64,
                            scale: f64,
                            c: f64,
                            bc: f64,
                            sz: f64,
                            mut f: impl FnMut(f64, usize, usize),
                        ) {
                            let mut index = 0;
                            let mut value = ((((-bc / scale) + c) / (step * 10.0)).floor() - 1.0)
                                * (step * 10.0);
                            loop {
                                value += step;
                                index += 1;
                                let coord = (value - c) * scale + bc;
                                if coord <= 0.0 {
                                    continue;
                                }
                                if coord >= sz - 1.0 {
                                    break;
                                }
                                f(value, coord as usize, index);
                            }
                        }

                        each_grid_line(
                            grid_step,
                            scale,
                            center.x,
                            buf_center.x,
                            buf_f64.x,
                            |_, coord, index| {
                                if index % 10 == 0 {
                                    for l in buf.lines_mut(..) {
                                        l[coord] = cad_color_theme.grid_color_base;
                                    }
                                } else {
                                    for l in buf.lines_mut(..) {
                                        l[coord] = cad_color_theme.grid_color_sub;
                                    }
                                }
                            },
                        );

                        each_grid_line(
                            grid_step,
                            scale,
                            center.y,
                            buf_center.y,
                            buf_f64.y,
                            |_, coord, index| {
                                if index % 10 == 0 {
                                    for l in &mut buf[coord] {
                                        *l = cad_color_theme.grid_color_base;
                                    }
                                } else {
                                    for l in &mut buf[coord] {
                                        *l = cad_color_theme.grid_color_sub;
                                    }
                                }
                            },
                        );

                        each_grid_line(
                            grid_step,
                            scale,
                            center.x,
                            buf_center.x,
                            buf_f64.x,
                            |value, coord, index| {
                                if index % 10 == 0 {
                                    self.font
                                        .color(cad_color_theme.grid_font)
                                        .layout_vertical(TextLayoutVertical::TOP)
                                        .layout_horizontal(TextLayoutHorizontal::MIDDLE)
                                        .draw(&format!("{value}"), (coord as i32, 0), buf);
                                }
                            },
                        );

                        each_grid_line(
                            grid_step,
                            scale,
                            center.y,
                            buf_center.y,
                            buf_f64.y,
                            |value, coord, index| {
                                if index % 10 == 0 {
                                    self.font
                                        .color(cad_color_theme.grid_font)
                                        .layout_vertical(TextLayoutVertical::MIDDLE)
                                        .layout_horizontal(TextLayoutHorizontal::LEFT)
                                        .draw(&format!("{value}"), (0, coord as i32), buf);
                                }
                            },
                        );
                    }
                    let highlight_point = document.get_highlight_point();
                    let mut span_buffer = vec![(0, 0); buf.get_size().1 * 4];
                    for (id, element) in document.get_content() {
                        let locc = match element {
                            Element::LoCC(locc) => locc,
                            _ => continue,
                        };

                        let mut l = locc.locc;
                        l = l.translate(center.neg());
                        l = l.scale(scale);
                        l = l.translate(buf_center);

                        let width: f64 = if locc.selected { 3.0 } else { 1.0 };
                        let mut highlight = document.is_highlight(*id);
                        if let HighlightPointKind::Center(center_arc_id) = highlight_point.kind {
                            if center_arc_id == *id {
                                highlight = true;
                            }
                        }

                        match config.curves_aa_mode {
                            CurvesAAMode::NoAntiAliasing => draw_locc(
                                buf,
                                &l,
                                if highlight {
                                    cad_color_theme.highlight_line_color
                                } else {
                                    cad_color_theme.line_color
                                },
                                width,
                                &mut span_buffer,
                                1,
                            ),
                            CurvesAAMode::AntiAliasingX2 => draw_locc(
                                buf,
                                &l,
                                if highlight {
                                    cad_color_theme.highlight_line_aa_color
                                } else {
                                    cad_color_theme.line_aa_color
                                },
                                width,
                                &mut span_buffer,
                                2,
                            ),
                            CurvesAAMode::AntiAliasingX4 => draw_locc(
                                buf,
                                &l,
                                if highlight {
                                    cad_color_theme.highlight_line_aa_color
                                } else {
                                    cad_color_theme.line_aa_color
                                },
                                width,
                                &mut span_buffer,
                                4,
                            ),
                        };
                    }

                    let mut draw_pic = |position: Point<f64>, pic: &ImageView<bool>| {
                        let pic_size = pic.get_size();
                        let buf_size = buf.get_size();
                        let shift_x = pic_size.0 as i32 / 2;
                        let shift_y = pic_size.1 as i32 / 2;
                        if position.x >= -shift_x as f64
                            && position.x <= buf_size.0 as f64 - shift_x as f64 - 1.0
                            && position.y >= -shift_y as f64
                            && position.y <= buf_size.1 as f64 - shift_y as f64 - 1.0
                        {
                            buf.draw(
                                pic,
                                (position.x as i32 - shift_x, position.y as i32 - shift_y),
                                |dst, src| {
                                    if *src {
                                        *dst = cad_color_theme.pic_color;
                                    }
                                },
                            );
                        }
                    };
                    match highlight_point.kind {
                        HighlightPointKind::Grid => {
                            let pic_center =
                                (highlight_point.position - center).scale(scale) + buf_center;
                            draw_pic(pic_center, &self.picts.borrow().grid_point.as_view());
                        }
                        HighlightPointKind::End => {
                            let pic_center =
                                (highlight_point.position - center).scale(scale) + buf_center;
                            draw_pic(pic_center, &self.picts.borrow().end_point.as_view());
                        }
                        HighlightPointKind::Center(_) => {
                            let pic_center =
                                (highlight_point.position - center).scale(scale) + buf_center;
                            draw_pic(pic_center, &self.picts.borrow().center_point.as_view());
                        }
                        _ => {}
                    }

                    if let Some((c1, c2)) = document.get_selection_rectangle() {
                        let c1 = (c1 - center).scale(scale) + buf_center;
                        let c2 = (c2 - center).scale(scale) + buf_center;

                        let bounded1 = Point::new(
                            f64::max(-1.0, f64::min(c1.x, c2.x)),
                            f64::max(-1.0, f64::min(c1.y, c2.y)),
                        );
                        let bounded2 = Point::new(
                            f64::min((buf.get_size().0 + 1) as f64, f64::max(c1.x, c2.x)),
                            f64::min((buf.get_size().1 + 1) as f64, f64::max(c1.y, c2.y)),
                        );
                        if bounded1.x < bounded2.x && bounded1.y < bounded2.y {
                            if bounded1.x >= 0.0 && bounded1.y + 1.0 < bounded2.y - 1.0 {
                                buf.window_mut(
                                    ((bounded1.x) as usize, (bounded1.y + 1.0) as usize),
                                    ((bounded1.x + 1.0) as usize, (bounded2.y - 1.0) as usize),
                                )
                                .fill(|p| *p = cad_color_theme.selection_bevel_color);
                            }

                            if bounded2.x <= buf.get_size().0 as f64
                                && bounded1.y + 1.0 < bounded2.y - 1.0
                            {
                                buf.window_mut(
                                    ((bounded2.x - 1.0) as usize, (bounded1.y + 1.0) as usize),
                                    ((bounded2.x) as usize, (bounded2.y - 1.0) as usize),
                                )
                                .fill(|p| *p = cad_color_theme.selection_bevel_color);
                            }

                            if bounded1.y >= 0.0 && bounded1.x + 1.0 < bounded2.x - 1.0 {
                                buf.window_mut(
                                    ((bounded1.x + 1.0) as usize, (bounded1.y) as usize),
                                    ((bounded2.x - 1.0) as usize, (bounded1.y + 1.0) as usize),
                                )
                                .fill(|p| *p = cad_color_theme.selection_bevel_color);
                            }

                            if bounded2.y <= buf.get_size().1 as f64
                                && bounded1.x + 1.0 < bounded2.x - 1.0
                            {
                                buf.window_mut(
                                    ((bounded1.x + 1.0) as usize, (bounded2.y - 1.0) as usize),
                                    ((bounded2.x - 1.0) as usize, (bounded2.y) as usize),
                                )
                                .fill(|p| *p = cad_color_theme.selection_bevel_color);
                            }

                            if bounded1.x + 1.0 < bounded2.x - 1.0
                                && bounded1.y + 1.0 < bounded2.y - 1.0
                            {
                                buf.window_mut(
                                    ((bounded1.x + 1.0) as usize, (bounded1.y + 1.0) as usize),
                                    ((bounded2.x - 1.0) as usize, (bounded2.y - 1.0) as usize),
                                )
                                .fill(|p| {
                                    *p = *p - ((*p & 0xFCFCFC) >> 2)
                                        + cad_color_theme.selection_rect_color
                                });
                            }
                        }
                    }
                }

                return true;
            }
            GuiMessage::MouseDown(position) => {
                let (_, document_position) = self.screen_coord_to_document_coord(Point::new(
                    position.0 as f64,
                    position.1 as f64,
                ));

                let mut document = self.document.borrow_mut();
                document.l_button_down(document_position);

                return true;
            }
            GuiMessage::MouseUp(position, _) => {
                let (_, document_position) = self.screen_coord_to_document_coord(Point::new(
                    position.0 as f64,
                    position.1 as f64,
                ));

                let mut document = self.document.borrow_mut();
                document.l_button_up(document_position);

                return true;
            }
            GuiMessage::MouseMove(position) => {
                let (_, document_position) = self.screen_coord_to_document_coord(Point::new(
                    position.0 as f64,
                    position.1 as f64,
                ));

                let mut document = self.document.borrow_mut();
                return document.mouse_move(document_position, &self.config.borrow());
            }
            _ => return false,
        }
    }
}
