use speedy2d::{Graphics2D, Window};
use speedy2d::color::Color;
use speedy2d::dimen::Vec2;
use speedy2d::font::{Font, TextLayout, TextOptions};
use speedy2d::window::{KeyScancode, MouseButton, VirtualKeyCode, WindowHandler, WindowHelper};

use crate::clipping::clip_polygon;
use crate::edge::Edge;
use crate::polygon::{is_point_in_polygon, is_polygon_clockwise};
use crate::ui::UiState::WaitSubject;

const EDGE_THICKNESS: f32 = 3.0;
const CANDIDATE_THICKNESS: f32 = 1.0;
const WIDTH: u32 = 1600;
const HEIGHT: u32 = 900;

#[derive(PartialEq)]
enum UiState {
    WaitSubject,
    WaitClipping,
    InputDone
}

pub struct UiLogic {
    state: UiState,
    subject_polygon: Vec<Edge>,
    clipping_polygon: Vec<Edge>,
    result_polygon: Vec<Edge>,
    new_polygon_part: Vec<Vec2>,
    cursor: Vec2,
    font: Font
}

impl Default for UiLogic {
    fn default() -> Self {
        let font_bytes = include_bytes!("../assets/LiberationSans-Regular.ttf");
        UiLogic {
            state: WaitSubject,
            subject_polygon: vec![],
            clipping_polygon: vec![],
            result_polygon: vec![],
            new_polygon_part: vec![],
            cursor: Vec2 { x: 0.0, y: 0.0 },
            font: Font::new(font_bytes).expect("Font loading failed")
        }
    }
}

fn draw_polygon_with_hint(points: &Vec<Vec2>, cursor: &Vec2, color: Color, graphics: &mut Graphics2D) {
    for (i, point) in points.iter().enumerate() {
        if i == points.len() - 1 { break; }
        graphics.draw_line(point, points[i + 1], EDGE_THICKNESS, color)
    }
    if !points.is_empty() {
        graphics.draw_line(points.last().unwrap(), cursor, CANDIDATE_THICKNESS, color)
    }
}

fn draw_polygon(edges: &Vec<Edge>, thickness: f32, color: Color, graphics: &mut Graphics2D) {
    if edges.is_empty() { return }
    for edge in edges.iter() {
        graphics.draw_line(edge.from, edge.to, thickness, color)
    }
}

fn point_vec_to_edges(points: &Vec<Vec2>) -> Vec<Edge> {
    if points.len() < 3 {
        panic!("Malformed polygon!")
    }
    let mut ret: Vec<Edge> = vec![];
    for (i, point) in points.iter().enumerate() {
        if i == points.len() - 1 { break; }
        ret.push(Edge { from: *point, to: points[i + 1] })
    }
    ret.push(Edge { from: *points.last().unwrap(), to: points[0] });
    ret
}

fn draw_state_text(text: &str, font: &Font, graphics: &mut Graphics2D) {
    let blk = font.layout_text(text, 32.0, TextOptions::new());
    graphics.draw_text((50.0, 50.0), Color::BLACK, &blk)
}

fn draw_grid(graphics: &mut Graphics2D) {
    for i in 1..100 {
        let j = i as f32;
        graphics.draw_line(&Vec2{x: j * 100.0, y: 0.0},
                           &Vec2 {x: j * 100.0, y: 5000.0},
                           1.0, Color::LIGHT_GRAY);
        graphics.draw_line(&Vec2{y: j * 100.0, x: 0.0},
                           &Vec2 {y: j * 100.0, x: 5000.0},
                           1.0, Color::LIGHT_GRAY);
    }
}

fn draw_result_overlay(result: &[Edge], subject: &[Edge], clipping: &[Edge], graphics: &mut Graphics2D) {
    for i in (1..WIDTH).step_by(5) {
        for j in (1..HEIGHT).step_by(5) {
            let point = Vec2 { x: (i as f32), y: (j as f32) };
            if ! is_point_in_polygon(point, result)
                || ! is_point_in_polygon(point, subject)
                || ! is_point_in_polygon(point, clipping) {
                continue;
            }
            graphics.draw_circle(point, 2.0, Color::CYAN);
        }
    }
}

impl WindowHandler for UiLogic {
    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D) {
        graphics.clear_screen(Color::WHITE);

        draw_grid(graphics);
        match self.state {
            WaitSubject => {
                draw_state_text("Input Subject Polygon (RED)\n(Esc=Undo, RightMB=Close Curve, Enter=Next Step)", &self.font, graphics);
                draw_polygon_with_hint(&self.new_polygon_part, &self.cursor, Color::RED, graphics);
            },
            UiState::WaitClipping => {
                draw_state_text("Input Clipping Polygon (GREEN)\n(Esc=Undo, RightMB=Close Curve, Enter=Next Step)", &self.font, graphics);
                draw_polygon_with_hint(&self.new_polygon_part, &self.cursor, Color::GREEN, graphics);
            },
            UiState::InputDone => {
                draw_state_text("Result (BLUE)\n(Enter = Clear)", &self.font, graphics);
            }
        }
        draw_polygon(&self.subject_polygon, EDGE_THICKNESS, Color::RED, graphics);
        draw_polygon(&self.clipping_polygon, EDGE_THICKNESS,Color::GREEN, graphics);
        if self.result_polygon.len() >= 3 {
            draw_result_overlay(&self.result_polygon, &self.subject_polygon, &self.clipping_polygon, graphics);
        }
        draw_polygon(&self.result_polygon, EDGE_THICKNESS, Color::BLUE, graphics);

        helper.request_redraw()
    }

    fn on_mouse_move(&mut self, _helper: &mut WindowHelper<()>, position: Vec2) {
        self.cursor = position;
    }

    fn on_mouse_button_up(&mut self, _helper: &mut WindowHelper<()>, button: MouseButton) {
        match button {
            MouseButton::Left => {
                if self.state != UiState::InputDone {
                    if let Some(x) = self.new_polygon_part.last() {
                        if (x - self.cursor).magnitude_squared() < 1.0 {
                            return; // avoid malformed edges
                        }
                    }
                    self.new_polygon_part.push(self.cursor)
                }
            }
            MouseButton::Right => {
                if self.new_polygon_part.len() < 3 { return; }
                let target = match self.state {
                    WaitSubject => { &mut self.subject_polygon },
                    UiState::WaitClipping => { &mut self.clipping_polygon }
                    _ => { return; }
                };
                let np = &self.new_polygon_part;
                let poly: Vec<Edge> = np.iter().zip(
                    np.iter().skip(1).chain(np.iter().take(1))
                ).map(|(p1, p2)| Edge { from: *p1, to: *p2 }).collect();
                if target.is_empty() ^ is_polygon_clockwise(&poly) {
                    target.append(&mut point_vec_to_edges(&self.new_polygon_part));
                }
                self.new_polygon_part.clear();
            }
            MouseButton::Middle => {
                self.cancel_current_polygon();
            }
            _ => {}
        }
    }

    fn on_key_down(&mut self, _helper: &mut WindowHelper<()>, virtual_key_code: Option<VirtualKeyCode>, _scancode: KeyScancode) {
        match virtual_key_code {
            None => {}
            Some(keycode) => {
                if keycode == VirtualKeyCode::Return {
                    if ! self.new_polygon_part.is_empty() { return }
                    match self.state {
                        WaitSubject => {
                            if self.new_polygon_part.len() < 3 && !self.new_polygon_part.is_empty() { return; }
                            self.state = UiState::WaitClipping
                        },
                        UiState::WaitClipping => {
                            if self.new_polygon_part.len() < 3 && !self.new_polygon_part.is_empty() { return; }
                            self.result_polygon = clip_polygon(&self.subject_polygon, &self.clipping_polygon);
                            self.state = UiState::InputDone
                        },
                        UiState::InputDone => {
                            self.clipping_polygon.clear();
                            self.subject_polygon.clear();
                            self.result_polygon.clear();
                            self.state = WaitSubject
                        }
                    }
                } else if keycode == VirtualKeyCode::Escape {
                    self.cancel_current_polygon()
                }
            }
        }
    }
}

impl UiLogic {
    fn cancel_current_polygon(&mut self) {
        if self.new_polygon_part.is_empty() {
            self.state = UiState::WaitSubject;
            self.subject_polygon.clear();
            self.clipping_polygon.clear();
            self.result_polygon.clear();
        } else {
            self.new_polygon_part.clear();
        }
    }
}

pub fn run_loop() {
    let win = Window::new_centered("Polygon Clipping", (WIDTH, HEIGHT)).expect("Window creation failed");
    win.run_loop(UiLogic::default())
}