use crate::config::*;
use curves::solver::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Group {
    pub ids: HashSet<usize>,
    pub selected: bool,
}

type Point = curves::points::Point<f64>;
type Contour = curves::Contour<f64>;
type Segment = curves::Segment<f64>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Curve {
    pub curve: curves::Curve<f64>,
    pub group_id: Option<usize>,
    pub selected: bool,
}

impl Curve {
    pub fn new_contour(c: Contour) -> Self {
        Self {
            curve: curves::Curve::Contour(c),
            group_id: None,
            selected: false,
        }
    }

    pub fn new_segment(s: Segment) -> Self {
        Self {
            curve: curves::Curve::Segment(s),
            group_id: None,
            selected: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Element {
    Curve(Curve),
    Group(Group),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum HighlightPointKind {
    None,
    End,
    Cross,
    Grid,
    Center(usize),
    Tangent(usize),
    Normal(usize),
}

impl Default for HighlightPointKind {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct HighlightPoint {
    pub position: Point,
    pub kind: HighlightPointKind,
}

impl HighlightPoint {
    pub fn end(position: Point) -> Self {
        Self {
            position,
            kind: HighlightPointKind::End,
        }
    }

    pub fn cross(position: Point) -> Self {
        Self {
            position,
            kind: HighlightPointKind::Cross,
        }
    }

    pub fn grid(position: Point) -> Self {
        Self {
            position,
            kind: HighlightPointKind::Grid,
        }
    }

    pub fn center(position: Point, id: usize) -> Self {
        Self {
            position,
            kind: HighlightPointKind::Center(id),
        }
    }

    pub fn tangent(position: Point, id: usize) -> Self {
        Self {
            position,
            kind: HighlightPointKind::Tangent(id),
        }
    }

    pub fn normal(position: Point, id: usize) -> Self {
        Self {
            position,
            kind: HighlightPointKind::Normal(id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Edition {
    Add(Element, usize),
    Remove(Element, usize),
    AddToGroup(usize, usize),
    RemoveFromGroup(usize, usize),
}

enum EditionRef<'i> {
    Add(&'i Element, usize),
    Remove(&'i Element, usize),
    AddToGroup(usize, usize),
    RemoveFromGroup(usize, usize),
}

impl<'i> Edition {
    fn redo(&'i self) -> EditionRef<'i> {
        match self {
            Self::Add(e, id) => EditionRef::Add(&e, *id),
            Self::Remove(e, id) => EditionRef::Remove(&e, *id),
            Self::AddToGroup(g, id) => EditionRef::AddToGroup(*g, *id),
            Self::RemoveFromGroup(g, id) => EditionRef::RemoveFromGroup(*g, *id),
        }
    }

    fn undo(&'i self) -> EditionRef<'i> {
        match self {
            Self::Add(e, id) => EditionRef::Remove(&e, *id),
            Self::Remove(e, id) => EditionRef::Add(&e, *id),
            Self::AddToGroup(g, id) => EditionRef::RemoveFromGroup(*g, *id),
            Self::RemoveFromGroup(g, id) => EditionRef::AddToGroup(*g, *id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Diff {
    editions: Vec<Edition>,
}

impl Diff {
    fn append(mut self, mut other: Diff) -> Self {
        self.editions.append(&mut other.editions);
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct DocumentSelecting {
    corner1: Point,
    corner2: Point,
    selected_ids: HashSet<usize>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct DocumentClick {
    point: Point,
    selected_id: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
enum DocumentState {
    Nothing,
    DocumentClick(DocumentClick),
    DocumentSelecting(DocumentSelecting),
}

impl Default for DocumentState {
    fn default() -> Self {
        Self::Nothing
    }
}

static EPS: f64 = 1.0e-12;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Document {
    content: HashMap<usize, Element>,
    history: Vec<Diff>,
    history_position: usize,
    last_entity_id: usize,

    center: Point,
    scale: i32,
    state: DocumentState,

    #[serde(skip)]
    highliht_id: Option<usize>,
    #[serde(skip)]
    highlight_point: HighlightPoint,
}

impl Document {
    pub fn new() -> Self {
        Self {
            scale: 300,
            ..Default::default()
        }
    }

    pub fn set_center(&mut self, center: Point) {
        self.center = center;
    }

    pub fn get_center(&self) -> Point {
        self.center
    }

    pub fn get_scale(&self) -> f64 {
        f64::powi(1.01, self.scale)
    }

    pub fn change_scale(&mut self, delta: i32) {
        self.scale += delta;
        self.scale = std::cmp::min(std::cmp::max(self.scale, -1000), 1500);
    }

    pub fn get_content(&self) -> &HashMap<usize, Element> {
        &self.content
    }

    pub fn get_selection_rectangle(&self) -> Option<(Point, Point)> {
        if let DocumentState::DocumentSelecting(documelt_selecting) = &self.state {
            Some((documelt_selecting.corner1, documelt_selecting.corner2))
        } else {
            None
        }
    }

    pub fn is_highlight(&self, id: usize) -> bool {
        self.highliht_id.map(|hid| hid == id).unwrap_or(false)
    }

    pub fn get_highlight_point(&self) -> &HighlightPoint {
        &self.highlight_point
    }

    fn apply_diff(content: &mut HashMap<usize, Element>, diff: &Diff) {
        for edition in &diff.editions {
            Self::apply_edition(content, edition.redo());
        }
    }

    fn add_and_apply_diff(&mut self, diff: Diff) {
        Self::apply_diff(&mut self.content, &diff);
        self.history.truncate(self.history_position);
        self.history.push(diff);
        self.history_position += 1;
    }

    pub fn fix_history(&mut self) {
        self.history.clear();
        self.history_position = 0;
    }

    pub fn undo(&mut self) {
        if self.history_position > 0 {
            self.history_position -= 1;
            for edition in &self.history[self.history_position].editions {
                Self::apply_edition(&mut self.content, edition.undo());
            }
        }
    }

    pub fn redo(&mut self) {
        if self.history_position < self.history.len() {
            Self::apply_diff(&mut self.content, &self.history[self.history_position]);
            self.history_position += 1;
        }
    }

    fn apply_edition(content: &mut HashMap<usize, Element>, edition: EditionRef) {
        // here we assume than removing group is empty, because elements was removed in another editions
        match edition {
            EditionRef::Add(element, id) => {
                content.insert(id, element.clone());
            }
            EditionRef::Remove(_, id) => {
                content.remove(&id);
            }
            EditionRef::AddToGroup(group_id, id) => {
                match content.get_mut(&id) {
                    Some(Element::Curve(e)) => {
                        e.group_id = Some(group_id);
                    }
                    _ => {}
                }
                match content.get_mut(&group_id) {
                    Some(Element::Group(g)) => {
                        g.ids.insert(group_id);
                    }
                    _ => {}
                }
            }
            EditionRef::RemoveFromGroup(group_id, id) => {
                match content.get_mut(&id) {
                    Some(Element::Curve(e)) => {
                        // assume than entity_group_id == Some(group_id)
                        // we cant assert here, because file can be corrupted
                        // TODO implement warning message system
                        e.group_id = None;
                    }
                    _ => {}
                }
                match content.get_mut(&group_id) {
                    Some(Element::Group(g)) => {
                        g.ids.remove(&group_id);
                    }
                    _ => {}
                }
            }
        }
    }

    fn remove_entity_diff(&self, id: usize) -> (Diff, Option<usize>) {
        // create diff only
        let mut diff = Diff::default();
        let mut result_group_id = None;

        if let Some(removed) = self.content.get(&id) {
            match removed {
                Element::Group(g) => {
                    for id in &g.ids {
                        if let Some(group_element) = self.content.get(id) {
                            diff.editions
                                .push(Edition::Remove(group_element.clone(), *id));
                        }
                    }
                }
                Element::Curve(e) => {
                    if let Some(group_id) = e.group_id {
                        result_group_id = Some(group_id);
                        diff.editions.push(Edition::RemoveFromGroup(group_id, id));
                    }
                }
            }
            diff.editions.push(Edition::Remove(removed.clone(), id));
        }

        (diff, result_group_id)
    }

    fn add_entity_diff(&self, curve: Curve) -> (Diff, usize) {
        // todo create diff only
        let entity_id = self.last_entity_id;
        let mut diff = Diff::default();
        diff.editions
            .push(Edition::Add(Element::Curve(curve), entity_id));

        (diff, entity_id)
    }

    pub fn add_entity(&mut self, curve: Curve) {
        let diff = self.add_entity_diff(curve).0;
        self.last_entity_id += 1;
        self.add_and_apply_diff(diff);
    }

    pub fn remove_selected(&mut self) {
        let mut diff = Diff::default();
        for (id, l) in &self.content {
            if let Element::Curve(curve) = l {
                if curve.selected {
                    diff = diff.append(self.remove_entity_diff(*id).0);
                }
            };
        }
        self.add_and_apply_diff(diff);
    }

    pub fn get_grid_step(&self) -> f64 {
        let mut grid_step = 1.0;
        let scale = self.get_scale();
        while grid_step * scale < 8.0 {
            grid_step *= 10.0;
        }
        grid_step
    }

    pub fn snap_distance(&self) -> f64 {
        20.0 / self.get_scale()
    }

    pub fn slide_distance(&self) -> f64 {
        5.0 / self.get_scale()
    }

    pub fn change_highlight_distance(&self) -> f64 {
        0.1 / self.get_scale()
    }

    pub fn l_button_down(&mut self, position: Point) {
        match &self.state {
            DocumentState::Nothing => {
                let max_distance = self.snap_distance();
                let target = self.find_nearest_locc(position, max_distance);
                self.state = DocumentState::DocumentClick(DocumentClick {
                    point: position,
                    selected_id: target,
                });
                if let Some(target) = target {
                    if let Some(Element::Curve(curve)) = self.content.get_mut(&target) {
                        curve.selected = !curve.selected;
                    }
                }
            }
            _ => {}
        }
    }

    fn generate_document_selecting(&self, corner1: Point, corner2: Point) -> DocumentSelecting {
        DocumentSelecting {
            corner1,
            corner2,
            selected_ids: self.find_locc_inside_rect(corner1, corner2),
        }
    }

    fn set_selection(&mut self, ids: &HashSet<usize>, selected: bool) {
        for id in ids {
            if let Some(Element::Curve(curve)) = self.content.get_mut(id) {
                curve.selected = selected;
            }
        }
    }

    fn fill_snap_point_info(&mut self, position: Point, config: &Config) -> bool {
        let mut new_highlight_point = HighlightPoint::default();
        let mut sqr_dist = self.snap_distance() * self.snap_distance();
        let treshold = EPS;
        // step1: try snap to grid
        if config.snap_options.snap_grid {
            let grid_step = self.get_grid_step();
            let x1 = (position.x / grid_step).floor() * grid_step;
            let x2 = x1 + grid_step;
            let y1 = (position.y / grid_step).floor() * grid_step;
            let y2 = y1 + grid_step;
            let grid_point = Point::new(
                if position.x - x1 > x2 - position.x {
                    x2
                } else {
                    x1
                },
                if position.y - y1 > y2 - position.y {
                    y2
                } else {
                    y1
                },
            );
            let sqr_candidate_dist = (position - grid_point).sqr_length();
            if sqr_candidate_dist < sqr_dist - treshold {
                sqr_dist = sqr_candidate_dist;
                new_highlight_point = HighlightPoint::grid(grid_point);
            }
        }
        // step2 : try snap to endpoint
        let mut iter1 = self.content.iter();
        while let Some((id, l)) = iter1.next() {
            if let Element::Curve(curve) = l {
                if config.snap_options.snap_endpoints {
                    if let curves::Curve::Segment(s) = curve.curve {
                        let sqr_candidate_dist = (position - s.begin).sqr_length();
                        if sqr_candidate_dist < sqr_dist - treshold {
                            sqr_dist = sqr_candidate_dist;
                            new_highlight_point = HighlightPoint::end(s.begin);
                        }
                        let sqr_candidate_dist = (position - s.end).sqr_length();
                        if sqr_candidate_dist < sqr_dist - treshold {
                            sqr_dist = sqr_candidate_dist;
                            new_highlight_point = HighlightPoint::end(s.end);
                        }
                    }
                }
                if config.snap_options.snap_centers {
                    let contour = curve.curve.get_contour();
                    if let Some((sqr_candidate_dist, center)) =
                        contour.sqr_distance_to_center(position, sqr_dist)
                    {
                        if sqr_candidate_dist < sqr_dist - treshold {
                            sqr_dist = sqr_candidate_dist;
                            new_highlight_point = HighlightPoint::center(center, *id);
                        }
                    }
                }
                if config.snap_options.snap_crosses {
                    let dist_to_current = curve.curve.distance(position);
                    if dist_to_current * dist_to_current < sqr_dist {
                        let mut iter2 = iter1.clone();
                        while let Some((_id2, l2)) = iter2.next() {
                            if let Element::Curve(curve2) = l2 {
                                let dist_to_current2 = curve2.curve.distance(position);
                                if dist_to_current2 * dist_to_current2 < sqr_dist {
                                    for candidate in
                                        intersection_curves(&curve.curve, &curve2.curve, EPS)
                                    {
                                        let sqr_candidate_dist =
                                            (position - candidate).sqr_length();
                                        if sqr_candidate_dist < sqr_dist - treshold {
                                            sqr_dist = sqr_candidate_dist;
                                            new_highlight_point = HighlightPoint::cross(candidate);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            };
        }

        if self.highlight_point.kind != new_highlight_point.kind
            || (self.highlight_point.position - new_highlight_point.position).sqr_length()
                > self.change_highlight_distance() * self.change_highlight_distance()
        {
            self.highlight_point = new_highlight_point;
            true
        } else {
            false
        }
    }

    pub fn mouse_move(&mut self, position: Point, config: &Config) -> bool {
        let mut state = std::mem::take(&mut self.state); // prevent borrowing self
        match &mut state {
            DocumentState::Nothing => {
                let max_distance = self.snap_distance();
                let target = self.find_nearest_locc(position, max_distance);
                let result = self.highliht_id != target;
                self.highliht_id = target;
                let snap_point_changed = self.fill_snap_point_info(position, config);
                result || snap_point_changed
            }
            DocumentState::DocumentClick(document_click) => {
                let max_distance = self.slide_distance();
                if (document_click.point - position).sqr_length() > max_distance * max_distance {
                    if let Some(target) = document_click.selected_id {
                        if let Some(Element::Curve(curve)) = self.content.get_mut(&target) {
                            curve.selected = false;
                        }
                    }
                    let new_selection =
                        self.generate_document_selecting(document_click.point, position);
                    self.set_selection(&new_selection.selected_ids, true);
                    self.state = DocumentState::DocumentSelecting(new_selection);
                } else {
                    self.state = state;
                }
                true
            }
            DocumentState::DocumentSelecting(document_selecting) => {
                self.set_selection(&document_selecting.selected_ids, false);
                let new_selection =
                    self.generate_document_selecting(document_selecting.corner1, position);
                self.set_selection(&new_selection.selected_ids, true);
                self.state = DocumentState::DocumentSelecting(new_selection);
                true
            }
        }
    }

    pub fn l_button_up(&mut self, _: Point) {
        self.state = DocumentState::Nothing;
    }

    pub fn skip_state(&mut self) {
        self.state = DocumentState::Nothing;
        for (_, l) in &mut self.content {
            if let Element::Curve(curve) = l {
                curve.selected = false;
            };
        }
    }

    fn find_nearest_locc(&self, position: Point, max_distance: f64) -> Option<usize> {
        let mut max_distance = max_distance;
        let mut target = None;
        for (id, l) in &self.content {
            let curve = match l {
                Element::Curve(curve) => curve,
                _ => continue,
            };

            let dist = curve.curve.distance(position).abs();
            if dist < max_distance {
                max_distance = dist;
                target = Some(*id);
            }
        }

        target
    }

    fn find_locc_inside_rect(&self, corner1: Point, corner2: Point) -> HashSet<usize> {
        let mut result = HashSet::new();
        for (id, l) in &self.content {
            let curve = match l {
                Element::Curve(curve) if !curve.selected => curve,
                _ => continue,
            };

            if curve.curve.in_rect(corner1, corner2) {
                result.insert(*id);
            }
        }

        result
    }
}
