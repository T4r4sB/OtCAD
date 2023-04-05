use std::cell::RefCell;
use std::rc::Rc;

use application::gui::*;

use curves::points::*;
use curves::render::*;
use curves::*;

mod cells;

use cells::*;
use window::*;

pub struct Surface {
    base: GuiControlBase,
    time: std::time::SystemTime,
    cells: CellSet,
    state: PartitionState,
    scale: f32,
    direct_factor: f32,
}

impl std::fmt::Debug for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("Surface")
    }
}

impl Surface {
    pub fn new(size_constraints: SizeConstraints) -> Self {
        let mut cells = CellSet::new(
            // 20,
            // &[
            //     Cell::new(&[(3, 1), (4, 1), (3, 2)]),
            //     Cell::new(&[(5, 1), (5, 2), (5, 1), (5, 2)]),
            //     Cell::new(&[(5, 1); 4]),
            //     Cell::new(&[(5, 2); 4]),
            //     Cell::new(&[(6, 1); 5]),
            //     Cell::new(&[(6, 2); 5]),
            //     Cell::new(&[(8, 1); 10]),
            //     Cell::new(&[(8, 2); 10]),
            // ],
            //6, &[Cell::new(&[(1, 1); 3])]
            // 12, &[
            //     Cell::new(&[(3, 1), (2, 1), (7, 1), (1, 1), (5, 1)]),
            // ],
            // 6,
            // &[
            //     Cell::new(&[(1, 1), (2, 1), (1, 1), (2, 1)]),
            // ],
            // 10,
            // &[
            //     Cell::new(&[(2, 1), (3, 1), (3, 1), (2, 2)]),
            //     Cell::new(&[(3, 1); 5]),
            //     Cell::new(&[(3, 2); 5]),
            //     Cell::new(&[(4, 1); 10]),
            //     Cell::new(&[(4, 2); 10]),
            //     Cell::new(&[(2, 2), (1, 2), (2, 1)]),
            // ],
            8,
            &[
                Cell::new(&[(3, 1); 8]),
                Cell::new(&[(2, 1), (2, 2), (3, 2), (3, 2), (3, 2), (3, 2)]),
                Cell::new(&[(3, 1), (3, 2), (2, 2), (3, 1), (3, 2), (2, 2)]),
                Cell::new(&[(1, 2), (1, 3), (2, 3)]),
            ],
            //  10, &[
            //      Cell::new(&[(1, 1), (7, 1), (1, 1), (3, 1), (3, 1)]),
            //  ],

            //  12, &[
            //     Cell::new(&[(3, 1); 4]),
            //      Cell::new(&[(2, 1); 3]),
            //       Cell::new(&[(4, 1); 6]),
            //     Cell::new(&[(5, 1); 12]),
            //   ],

            // 8, &[
            //     Cell::new(&[(1,1),(2,1),(1,2)]),
            //      Cell::new(&[(2, 1); 4]),
            //      Cell::new(&[(2, 2); 4]),
            //      Cell::new(&[(3, 1); 8]),
            //      Cell::new(&[(3, 2); 8]),
            //     ]

            //      16, &[
            //     Cell::new(&[(4, 1); 4]),
            //     Cell::new(&[(4, 2); 4]),
            //     Cell::new(&[(3, 1), (2, 1), (3, 2)]),
            // ],
        )
        .unwrap();

        let state = cells.init_partition_state().unwrap();

        Self {
            base: GuiControlBase::new(size_constraints),
            time: std::time::SystemTime::now(),
            cells,
            state,
            scale: 10.0,
            direct_factor: 1.0,
        }
    }
}

impl GuiControl for Surface {
    fn get_base_mut(&mut self) -> &mut GuiControlBase {
        &mut self.base
    }

    fn on_message(&mut self, m: GuiMessage) -> bool {
        match m {
            GuiMessage::MouseDown(_) => {
                self.state = self.cells.init_partition_state().unwrap();
                return true;
            }
            GuiMessage::MouseWheel(_, diff) => {
                if diff < 0 {
                    self.scale *= 1.1;
                } else {
                    self.scale /= 1.1;
                }

                return true;
            }
            GuiMessage::Char(code) => match code {
                '+' | '=' => {
                    self.direct_factor = f32::min(1.0, self.direct_factor + 0.1);
                    return true;
                }
                '-' => {
                    self.direct_factor = f32::max(0.0, self.direct_factor - 0.1);
                    return true;
                }
                _ => return false,
            },
            GuiMessage::Timer => {
                let mut elapsed = self.time.elapsed().unwrap();
                let dt = std::time::Duration::from_millis(1);
                let mut result = false;
                while elapsed > dt {
                    self.time += dt;
                    elapsed -= dt;

                    let mut total = 0;
                    for s in &self.state.items {
                        total += s.last_cells.len();
                    }

                    if total < 4000 {
                        for _ in 0..1 {
                            self.cells.iter_partition_state(&mut self.state);
                        }
                        result = true;
                    }
                }
                return result;
            }
            GuiMessage::Draw(buf, _theme, force) => {
                if self.base.can_draw(force) {
                    buf.fill(|p| *p = 0x000000);
                    let mut span_buffer = vec![(0, 0); buf.get_size().1 * 4];

                    let center =
                        Point::new(buf.get_size().0 as f32 * 0.5, buf.get_size().1 as f32 * 0.5);

                    let direct_color = (0x55 as f32 * self.direct_factor) as u32 * 0x000300;
                    let second_color = (0x55 as f32 * (1.0 - self.direct_factor)) as u32 * 0x010203;

                    for (c1, c2) in CellSet::get_direct_pairs(&self.state) {
                        let l1 = c1.scale(self.scale) + center;
                        let l2 = c2.scale(self.scale) + center;
                        let l = Curve::Segment(curves::Segment {
                            contour: Contour::<f32>::line(l1, l2),
                            begin: l1,
                            end: l2,
                            big: false,
                        });

                        if (l1 - l2).length() > 0.001 {
                            draw_locc(buf, &l, direct_color, 1.5, &mut span_buffer, 4);
                        }
                    }

                    if self.direct_factor < 1.0 {
                        for (c1, c2) in CellSet::get_pairs(&self.state) {
                            let l1 = c1.scale(self.scale) + center;
                            let l2 = c2.scale(self.scale) + center;
                            let l = Curve::Segment(curves::Segment {
                                contour: Contour::<f32>::line(l1, l2),
                                begin: l1,
                                end: l2,
                                big: false,
                            });

                            if (l1 - l2).length() > 0.001 {
                                draw_locc(buf, &l, second_color, 1.5, &mut span_buffer, 4);
                            }
                        }
                    }
                }

                return true;
            }
            _ => return false,
        }
    }
}

struct GuiTest {
    surface: Option<Rc<RefCell<Surface>>>,
}

impl GuiTest {
    fn new() -> Self {
        Self { surface: None }
    }
}

impl window::Application for GuiTest {
    fn on_create(&mut self, context: Rc<RefCell<window::Context>>) {
        self.surface = Some(context.borrow_mut().gui_system.set_root(Surface::new(
            SizeConstraints(SizeConstraint::flexible(100), SizeConstraint::flexible(100)),
        )));
    }

    fn on_close(&mut self, _context: Rc<RefCell<window::Context>>) {}

    fn on_change_position(&mut self, _window_position: WindowPosition) {}
}

fn main() {
    if let Err(_) = window::run_application("GuiTest", Box::new(GuiTest::new()), None) {
        // Do nothing, read message and exit
    }
}
