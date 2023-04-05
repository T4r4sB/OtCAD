use indexmap::{map::IndexMap, set::IndexSet};
use rand::rngs::*;
use rand::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use curves::points::*;

const EPS: f32 = 1.0e-6;

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
struct CellItem {
    angle: i32,
    segment: i32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Cell {
    items: Vec<CellItem>,
}

impl Cell {
    pub fn new(items: &[(i32, i32)]) -> Self {
        assert!(items.len() >= 3);
        for i in items {
            assert!(i.0 > 0);
            assert!(i.1 >= 0);
        }
        let items = items
            .iter()
            .map(|&(angle, segment)| CellItem { angle, segment })
            .collect();
        Self { items }
    }

    pub fn roll(mut self, count: i32) -> Self {
        if count < 0 {
            self.items.rotate_right(count as usize);
        } else {
            self.items.rotate_left(count as usize);
        }
        self
    }

    pub fn mirror(mut self) -> Self {
        self.items.reverse();
        let mut tmp = self.items.last().unwrap().angle;
        for i in &mut self.items {
            std::mem::swap(&mut i.angle, &mut tmp);
        }
        self
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
struct Angle {
    segments: (i32, i32),
    angle: i32,
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct Segment {
    pub segments: (i32, i32, i32),
    pub angles: (i32, i32),
}

type SegmentPartitionResult = IndexMap<Cell, (Option<usize>, Option<usize>)>;

pub struct CellSet {
    // parameters
    round_pieces: i32,
    possible_cells: IndexSet<Cell>,
    // must deduce it
    rng: ThreadRng,
    segment_lenghts: Vec<f32>,
    possible_angle_partitions: RefCell<IndexMap<Angle, usize>>,
    possible_segment_partitions: RefCell<IndexMap<Segment, Rc<SegmentPartitionResult>>>,
}

#[derive(Debug, Default)]
struct Equation {
    left: Vec<f32>,
    right: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct PlacedCellItem {
    coords: Point<f32>,
    angle: i32,
    segment: i32,
    direction: i32,
    pub edge_id: i32,
}

#[derive(Debug, Clone)]
pub struct PlacedCell {
    pub items: Vec<PlacedCellItem>,
}

impl PlacedCell {
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn get_coords(&self, index: usize) -> Point<f32> {
        self.items[index].coords
    }
}

#[derive(Debug)]
pub struct PartitionItem {
    pub last_cells: Vec<PlacedCell>,
    pub big_cell: PlacedCell,
    segment_index: usize,
    partitions: Rc<SegmentPartitionResult>,
    choices: Vec<usize>,
    edge_id: i32,
    same_edges: HashMap<i32, i32>,
}

#[derive(Debug)]
pub struct PartitionState {
    pub items: Vec<PartitionItem>,
}

impl CellSet {
    fn get_possible_lenghts(possible_cells: &[Cell]) -> Vec<bool> {
        let mut result = Vec::<bool>::new();
        for c in possible_cells {
            for i in &c.items {
                let s = i.segment as usize;
                if s >= result.len() {
                    result.resize(s + 1, false);
                }
                result[s] = true;
            }
        }

        result
    }

    fn angle_to_radians(angle_piece: i32, round_pieces: i32) -> f32 {
        (2.0 * std::f32::consts::PI) * angle_piece as f32 / round_pieces as f32
    }

    fn get_constraints(round_pieces: i32, cell: &Cell) -> Result<Vec<Equation>, String> {
        let mut angle_piece = 0;
        let mut v = Vec::new();

        for i in &cell.items {
            angle_piece += round_pieces / 2 - i.angle;
            let angle = Self::angle_to_radians(angle_piece, round_pieces);
            let si = i.segment as usize;
            if si >= v.len() {
                v.resize(si + 1, Point::new(0.0, 0.0));
            }
            v[si] += Point::angle(angle);
        }

        if angle_piece != round_pieces {
            return Err(format!("cell {:?} is bad: sum of angles incorrect", cell));
        }

        let mut eq: [Equation; 2] = Default::default();

        #[derive(Debug, Copy, Clone)]
        enum OnlyCoord {
            Zero,
            One(i32),
            Many,
        }

        fn apply(oc: OnlyCoord, c: i32) -> OnlyCoord {
            match oc {
                OnlyCoord::Zero => OnlyCoord::One(c),
                _ => OnlyCoord::Many,
            }
        }

        let mut cnt = [OnlyCoord::Zero; 2];
        for i in 0..v.len() {
            for j in 0..2 {
                if v[i][j].abs() > EPS {
                    if eq[j].left.len() <= i {
                        eq[j].left.resize(i + 1, 0.0);
                    }
                    eq[j].left[i] = v[i][j];
                    cnt[j] = apply(cnt[j], i as i32);
                }
            }
        }

        let mut result = Vec::with_capacity(2);

        for i in 0..2 {
            match cnt[i] {
                OnlyCoord::Zero => {}
                OnlyCoord::One(index) => {
                    return Err(format!(
                        "cell {:?} is bad: segment index {} deduced to be zero, \
                            because in only has non-zero sum of segments: {:?}",
                        cell, index, v[index as usize]
                    ))
                }
                OnlyCoord::Many => result.push(std::mem::take(&mut eq[i])),
            }
        }

        Ok(result)
    }

    fn solve(mut equations: Vec<Equation>, possible_lengths: &[bool]) -> Result<Vec<f32>, String> {
        for eq in &mut equations {
            eq.left.resize(possible_lengths.len(), 0.0);
        }

        let mut eqi = 0;
        for i in 0..possible_lengths.len() {
            if !possible_lengths[i] {
                continue;
            }

            let mut best: Option<usize> = None;
            let mut bestv = EPS;
            for besti in eqi..equations.len() {
                if equations[besti].left[i].abs() > bestv {
                    best = Some(besti);
                    bestv = equations[besti].left[i].abs();
                }
            }

            let best = best.ok_or(format!("cellset is wrong, because system of equations can non deduce length for segment {i}"))?;
            equations.swap(best, eqi);

            let delta = 1.0 / equations[eqi].left[i];
            equations[eqi].left[i] = 1.0;
            for j in i + 1..equations[eqi].left.len() {
                equations[eqi].left[j] *= delta;
            }
            equations[eqi].right *= delta;

            for aeq in 0..equations.len() {
                if aeq == eqi {
                    continue;
                }

                let k = equations[aeq].left[i];
                equations[aeq].left[i] = 0.0;

                for j in i + 1..equations[eqi].left.len() {
                    equations[aeq].left[j] -= equations[eqi].left[j] * k;
                }
                equations[aeq].right -= equations[eqi].right * k;
            }

            eqi += 1;
        }

        for eqi in eqi..equations.len() {
            if equations[eqi].left.iter().any(|k| k.abs() > EPS) || equations[eqi].right.abs() > EPS
            {
                return Err(format!(
                    "cellset is wrong, because system of equations contradicts itself"
                ));
            }
        }

        let mut result = vec![0.0; possible_lengths.len()];
        let mut eqi = 0;
        let mut min = 1.0;

        for i in 0..possible_lengths.len() {
            if !possible_lengths[i] {
                continue;
            }
            result[i] = equations[eqi].right;
            if result[i] < min {
                min = result[i];
            }
            if result[i] < EPS {
                return Err(format!(
                    "cellset is wrong, because length of segment {i} deduced to be negative"
                ));
            }
            eqi += 1;
        }

        for r in &mut result {
            *r /= min;
        }

        Ok(result)
    }

    pub fn new(round_pieces: i32, possible_cells: &[Cell]) -> Result<Self, String> {
        if round_pieces <= 2 || round_pieces % 2 != 0 {
            return Err(format!(
                "round_pieces {round_pieces} is bad: it should be even and > 2"
            ));
        }

        let possible_lengths = Self::get_possible_lenghts(&possible_cells);
        let mut equations = Vec::new();

        for c in possible_cells {
            equations.append(&mut Self::get_constraints(round_pieces, c)?);
        }

        let sum_equation = Equation {
            left: vec![1.0; possible_lengths.len()],
            right: 1.0,
        };
        equations.push(sum_equation);
        let segment_lenghts = Self::solve(equations, &possible_lengths)?;

        let mut possible_cells_set = IndexSet::new();

        for c in possible_cells {
            for i in 0..c.items.len() {
                let rotated_c = c.clone().roll(i as i32);
                let mirrored_c = rotated_c.clone().mirror();
                possible_cells_set.insert(rotated_c);
                possible_cells_set.insert(mirrored_c);
            }
        }

        Ok(Self {
            round_pieces,
            possible_cells: possible_cells_set,
            rng: thread_rng(),
            segment_lenghts,
            possible_angle_partitions: RefCell::new(IndexMap::new()),
            possible_segment_partitions: RefCell::new(IndexMap::new()),
        })
    }

    fn get_all_angle_partitions(&self, angle: Angle) -> usize {
        // TODO: fix it when cargo fix the borrow checker bug
        if let Some(result) = self.possible_angle_partitions.borrow().get(&angle).copied() {
            return result;
        }

        let mut result = 0;
        for s in &self.possible_cells {
            let fi = s.items.first().unwrap();
            let li = s.items.last().unwrap();
            if fi.segment != angle.segments.1 {
                continue;
            }
            if fi.angle > angle.angle {
                continue;
            }

            if fi.angle == angle.angle {
                if li.segment == angle.segments.0 {
                    result += 1;
                }
                continue;
            }

            let smaller_angle = Angle {
                angle: angle.angle - fi.angle,
                segments: (angle.segments.0, li.segment),
            };

            result += self.get_all_angle_partitions(smaller_angle);
        }

        self.possible_angle_partitions
            .borrow_mut()
            .insert(angle, result);
        result
    }

    pub fn get_all_segment_partitions(&self, segment: Segment) -> Rc<SegmentPartitionResult> {
        // TODO: fix it when cargo fix the borrow checker bug
        if let Some(result) = self
            .possible_segment_partitions
            .borrow()
            .get(&segment)
            .cloned()
        {
            return result;
        }

        let mut result = IndexMap::new();

        for s in &self.possible_cells {
            let fi = s.items.first().unwrap();
            let si = s.items[1];
            let li = s.items.last().unwrap();

            if fi.segment != segment.segments.1 {
                continue;
            }
            if fi.angle > segment.angles.0 || si.angle > segment.angles.1 {
                continue;
            }

            let r0 = if fi.angle < segment.angles.0 {
                let a = self.get_all_angle_partitions(Angle {
                    angle: segment.angles.0 - fi.angle,
                    segments: (segment.segments.0, li.segment),
                });
                if a == 0 {
                    continue;
                }
                Some(a)
            } else if segment.segments.0 == li.segment {
                None
            } else {
                continue;
            };

            let r1 = if si.angle < segment.angles.1 {
                let a = self.get_all_angle_partitions(Angle {
                    angle: segment.angles.1 - si.angle,
                    segments: (si.segment, segment.segments.2),
                });
                if a == 0 {
                    continue;
                }
                Some(a)
            } else if si.segment == segment.segments.2 {
                None
            } else {
                continue;
            };

            result.insert(s.clone(), (r0, r1));
        }

        let result = Rc::new(result);
        self.possible_segment_partitions
            .borrow_mut()
            .insert(segment, result.clone());
        result
    }

    pub fn place(
        &self,
        cell: &Cell,
        position: Point<f32>,
        direction: i32,
        edge_id: &mut i32,
    ) -> PlacedCell {
        let mut items = Vec::with_capacity(cell.items.len());

        let mut coords = position;
        let mut direction = direction;

        for i in &cell.items {
            if !items.is_empty() {
                direction = (direction + self.round_pieces / 2 - i.angle) % self.round_pieces;
            }

            items.push(PlacedCellItem {
                coords,
                angle: i.angle,
                segment: i.segment,
                direction,
                edge_id: *edge_id,
            });
            coords += Point::angle(CellSet::angle_to_radians(direction, self.round_pieces))
                .scale(self.segment_lenghts[i.segment as usize]);

            *edge_id += 1;
        }

        PlacedCell { items }
    }

    fn partition_item(
        &mut self,
        last_cells: Vec<PlacedCell>,
        big_cell: PlacedCell,
        edge_id: i32,
        same_edges: HashMap<i32, i32>,
    ) -> Option<PartitionItem> {
        let mut best: Option<(_, _, _)> = None;
        for i in 0..big_cell.items.len() {
            let prev = (i + big_cell.items.len() - 1) % big_cell.items.len();
            let next = (i + 1) % big_cell.items.len();
            let segment = Segment {
                angles: (
                    self.round_pieces - big_cell.items[next].angle,
                    self.round_pieces - big_cell.items[i].angle,
                ),
                segments: (
                    big_cell.items[next].segment,
                    big_cell.items[i].segment,
                    big_cell.items[prev].segment,
                ),
            };
            let results = self.get_all_segment_partitions(segment);
            if results.len() == 0 {
                return None;
            }

            let dist = (big_cell.items[i].coords + big_cell.items[next].coords).sqr_length();
            let max_dist = (2.0 * 40.0) * (2.0 * 40.0);
            if dist > max_dist {
                continue;
            }

            let criteria = dist * results.len() as f32;

            let need_change = if let Some((prev_d, _, _)) = best {
                criteria < prev_d - EPS
            } else {
                true
            };
            if need_change {
                best = Some((criteria, i, results.clone()));
            }
        }

        let best = best?;
        let choices = (0..best.2.len()).into_iter().collect();

        Some(PartitionItem {
            last_cells,
            big_cell,
            segment_index: best.1,
            partitions: best.2,
            choices,
            edge_id,
            same_edges,
        })
    }

    pub fn init_partition_state(&mut self) -> Option<PartitionState> {
        let cell = self
            .possible_cells
            .get_index(self.rng.gen_range(0..self.possible_cells.len()))
            .unwrap()
            .clone();
        //let position = Point::new(self.rng.gen_range(-1.0..1.0), self.rng.gen_range(-1.0..1.0));
        let position = Point::new(0.0, 0.0);
        let mut edge_id = 0;
        let placed_cell = self.place(&cell, position, 0, &mut edge_id);
        let partition = self.partition_item(
            vec![placed_cell.clone()],
            placed_cell,
            edge_id,
            HashMap::new(),
        )?;

        Some(PartitionState {
            items: vec![partition],
        })
    }

    fn concat(
        &self,
        cell1: &PlacedCell,
        index1: usize,
        cell2: &PlacedCell,
    ) -> Option<(PlacedCell, HashMap<i32, i32>)> {
        let mut items = Vec::with_capacity(cell1.items.len() + cell2.items.len() - 2);
        for i in 0..cell1.items.len() {
            if i == index1 {
                for j in 1..cell2.items.len() {
                    items.push(cell2.items[j]);
                }
            } else {
                items.push(cell1.items[i]);
            }
        }
        items[index1].angle += cell1.items[index1].angle;
        let second_merged_angle = (index1 + cell2.items.len() - 1) % items.len();
        items[second_merged_angle].angle += cell2.items[0].angle;

        let mut same_edges = HashMap::new();
        let e1 = cell1.items[index1].edge_id;
        let e2 = cell2.items[0].edge_id;
        same_edges.insert(e1, e2);
        same_edges.insert(e2, e1);

        'outer: loop {
            for i in 0..items.len() {
                let mut consumed = 0;
                loop {
                    if items[i].angle < self.round_pieces {
                        break;
                    }

                    if items[i].angle > self.round_pieces {
                        return None;
                    }

                    if items[i].angle == self.round_pieces
                        && items[(i + consumed) % items.len()].segment
                            != items[(i + items.len() - 1 - consumed) % items.len()].segment
                    {
                        return None;
                    }

                    let e1 = items[(i + consumed) % items.len()].edge_id;
                    let e2 = items[(i + items.len() - 1 - consumed) % items.len()].edge_id;
                    same_edges.insert(e1, e2);
                    same_edges.insert(e2, e1);

                    consumed += 1;
                    items[i].angle = items[(i + consumed) % items.len()].angle
                        + items[(i + items.len() - consumed) % items.len()].angle;
                }

                if consumed > 0 {
                    let angle_before_consume = (i + items.len() - consumed) % items.len();
                    let angle_after_consume = (i + consumed) % items.len();
                    items[angle_after_consume].angle = items[i].angle;
                    if angle_before_consume < angle_after_consume {
                        items.drain(angle_before_consume..angle_after_consume);
                    } else {
                        items = items[angle_after_consume..angle_before_consume].to_vec();
                    }

                    continue 'outer;
                }
            }

            break;
        }

        Some((PlacedCell { items }, same_edges))
    }

    pub fn iter_partition_state(&mut self, partition_state: &mut PartitionState) {
        loop {
            let last = if let Some(last) = partition_state.items.last_mut() {
                last
            } else {
                return;
            };
            if last.choices.len() == 0 {
                partition_state.items.pop();
                continue;
            }

            //let choice_index = self.rng.gen_range(0..last.choices.len());
            let choice_index = if last.choices.len() >= 2 {
                last.choices.len() - self.rng.gen_range(1..3)
            } else {
                0
            };
            let choice = last.choices.swap_remove(choice_index);

            let mut edge_id = last.edge_id;

            let cell = self.place(
                last.partitions.get_index(choice).unwrap().0,
                last.big_cell.items[(last.segment_index + 1) % last.big_cell.items.len()].coords,
                last.big_cell.items[last.segment_index].direction + self.round_pieces / 2,
                &mut edge_id,
            );

            if let Some((big_cell, same_edges)) =
                self.concat(&last.big_cell, last.segment_index, &cell)
            {
                if let Some(partition) =
                    self.partition_item(vec![cell.clone()], big_cell.clone(), edge_id, same_edges)
                {
                    partition_state.items.push(partition);
                    return;
                }
            }
            return;
        }
    }

    pub fn get_direct_pairs(state: &PartitionState) -> Vec<(Point<f32>, Point<f32>)> {
        let mut result = Vec::new();
        for item in &state.items {
            for c in &item.last_cells {
                for i in 0..c.len() {
                    result.push((c.items[i].coords, c.items[(i + 1) % c.len()].coords));
                }
            }
        }

        result
    }

    pub fn get_pairs(state: &PartitionState) -> Vec<(Point<f32>, Point<f32>)> {
        let mut id_to_coord = HashMap::new();
        let mut all_pairs = HashMap::new();
        for item in &state.items {
            for (k, v) in &item.same_edges {
                all_pairs.insert(*k, *v);
            }
            for c in &item.last_cells {
                let s = ((c.items[1].coords.x - c.items[0].coords.x)
                    * (c.items[2].coords.y - c.items[0].coords.y)
                    - (c.items[2].coords.x - c.items[0].coords.x)
                        * (c.items[1].coords.y - c.items[0].coords.y))
                    * 2.0;
                let xc = (c.items[0].coords.x * c.items[0].coords.x
                    + c.items[0].coords.y * c.items[0].coords.y)
                    * (c.items[1].coords.y - c.items[2].coords.y)
                    + (c.items[1].coords.x * c.items[1].coords.x
                        + c.items[1].coords.y * c.items[1].coords.y)
                        * (c.items[2].coords.y - c.items[0].coords.y)
                    + (c.items[2].coords.x * c.items[2].coords.x
                        + c.items[2].coords.y * c.items[2].coords.y)
                        * (c.items[0].coords.y - c.items[1].coords.y);
                let yc = (c.items[0].coords.x * c.items[0].coords.x
                    + c.items[0].coords.y * c.items[0].coords.y)
                    * (c.items[1].coords.x - c.items[2].coords.x)
                    + (c.items[1].coords.x * c.items[1].coords.x
                        + c.items[1].coords.y * c.items[1].coords.y)
                        * (c.items[2].coords.x - c.items[0].coords.x)
                    + (c.items[2].coords.x * c.items[2].coords.x
                        + c.items[2].coords.y * c.items[2].coords.y)
                        * (c.items[0].coords.x - c.items[1].coords.x);
                let center = Point::new(xc / s, -yc / s);

                for i in &c.items {
                    id_to_coord.insert(i.edge_id, center);
                }
            }
        }

        let mut used = HashSet::new();
        let mut result = Vec::new();
        for (i, c) in &id_to_coord {
            if used.contains(&i) {
                continue;
            }
            used.insert(i);
            if let Some(i2) = all_pairs.get(&i) {
                used.insert(i2);
                if let Some(c2) = id_to_coord.get(i2) {
                    result.push((*c, *c2));
                }
            }
        }

        result
    }
}
