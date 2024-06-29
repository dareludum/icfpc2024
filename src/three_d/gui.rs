use std::{thread, time};

use crate::{geometry::Vector2D, three_d::sim::ThreeDSimulator};

use super::{board::ThreeDBoard, sim::Cell};

use raylib::prelude::*;

#[derive(Debug, Default)]
struct GuiState {
    width: i32,
    height: i32,
    viewport_offset: Vector2D,
    viewport_drag_point: Option<Vector2>,
}

#[allow(dead_code)]
mod colors {
    use raylib::color::Color;

    // Solarized color palette (https://ethanschoonover.com/solarized)
    pub static SOLARIZED_BASE03: Color = Color::new(0x00, 0x2b, 0x36, 0xff);
    pub static SOLARIZED_BASE02: Color = Color::new(0x07, 0x36, 0x42, 0xff);
    pub static SOLARIZED_BASE01: Color = Color::new(0x58, 0x6e, 0x75, 0xff);
    pub static SOLARIZED_BASE00: Color = Color::new(0x65, 0x7b, 0x83, 0xff);
    pub static SOLARIZED_BASE0: Color = Color::new(0x83, 0x94, 0x96, 0xff);
    pub static SOLARIZED_BASE1: Color = Color::new(0x93, 0xa1, 0xa1, 0xff);
    pub static SOLARIZED_BASE2: Color = Color::new(0xee, 0xe8, 0xd5, 0xff);
    pub static SOLARIZED_BASE3: Color = Color::new(0xfd, 0xf6, 0xe3, 0xff);
    pub static SOLARIZED_YELLOW: Color = Color::new(0xb5, 0x89, 0x00, 0xff);
    pub static SOLARIZED_ORANGE: Color = Color::new(0xcb, 0x4b, 0x16, 0xff);
    pub static SOLARIZED_RED: Color = Color::new(0xdc, 0x32, 0x2f, 0xff);
    pub static SOLARIZED_MAGENTA: Color = Color::new(0xd3, 0x36, 0x82, 0xff);
    pub static SOLARIZED_VIOLET: Color = Color::new(0x6c, 0x71, 0xc4, 0xff);
    pub static SOLARIZED_BLUE: Color = Color::new(0x26, 0x8b, 0xd2, 0xff);
    pub static SOLARIZED_CYAN: Color = Color::new(0x2a, 0xa1, 0x98, 0xff);
    pub static SOLARIZED_GREEN: Color = Color::new(0x85, 0x99, 0x00, 0xff);
}

pub fn gui_main(board: ThreeDBoard, a: i64, b: i64) {
    const WINDOW_WIDTH: i32 = 1024;
    const WINDOW_HEIGHT: i32 = 768;

    let mut state = GuiState {
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        ..Default::default()
    };
    let mut sim = ThreeDSimulator::new(board, a, b);

    #[allow(unused_assignments)]
    let mut current_sim_result = Ok(None);

    let (mut rh, thread) = raylib::init().size(WINDOW_WIDTH, WINDOW_HEIGHT).build();
    while !rh.window_should_close() {
        {
            let mut d = rh.begin_drawing(&thread);
            d.clear_background(colors::SOLARIZED_BASE03);
            render_sim(&mut d, &state, &sim);
        }

        let mouse_pos = rh.get_mouse_position();

        if rh.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        } else if rh.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            state.viewport_drag_point = Some(mouse_pos);
        } else {
            // if mouse_pos.x < (rh.get_screen_width() as f32 - 50.0) {
            //     let scroll = rh.get_mouse_wheel_move();
            //     let scroll_amount = 0.95;
            //     if scroll.abs() > 0.5 {
            //         if scroll > 0.0 {
            //             let offset = 2.0 * state.translator.step / scroll_amount;
            //             state.translator.x_offset -= offset;
            //             state.translator.y_offset -= offset;
            //             state.translator.step /= scroll_amount;
            //         } else {
            //             let offset = 2.0 * state.translator.step * scroll_amount;
            //             state.translator.x_offset += offset;
            //             state.translator.y_offset += offset;
            //             state.translator.step *= scroll_amount;
            //         }
            //     }
            // }
        }

        if rh.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            // state.dragged_point = None;
            // state.rotate_pivot = None;
            // state.rotate_vertices_copy.clear();
            // if let Some(pos) = state.selection_pos {
            //     let rect = vec2_to_rect(pos, mouse_pos);
            //     let min = state.untranslate(&Vector2 {
            //         x: rect.x,
            //         y: rect.y,
            //     });
            //     let max = state.untranslate(&Vector2 {
            //         x: rect.x + rect.width,
            //         y: rect.y + rect.height,
            //     });
            //     let hits = hit_test_rect(&pose.borrow(), min, max);
            //     if !rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
            //         && !rh.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            //     {
            //         state.selected_points.clear();
            //     }
            //     if rh.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
            //         for hit in hits {
            //             state.selected_points.remove(&hit);
            //         }
            //     } else {
            //         for hit in hits {
            //             state.selected_points.insert(hit);
            //         }
            //     }
            //     state.selection_pos = None;
            // }
        } else if rh.is_mouse_button_released(MouseButton::MOUSE_BUTTON_RIGHT) {
            state.viewport_drag_point = None;
        }

        if rh.get_gesture_detected() == Gesture::GESTURE_DRAG
            || rh.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
            || rh.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT)
        {
            if let Some(p) = state.viewport_drag_point {
                let delta = mouse_pos - p;
                state.viewport_offset.x += delta.x as i32;
                state.viewport_offset.y += delta.y as i32;
                state.viewport_drag_point = Some(mouse_pos);
            }
        }

        let need_to_sleep = true;
        if let Some(key) = rh.get_key_pressed() {
            match key {
                KeyboardKey::KEY_A => {
                    sim.step_back();
                    current_sim_result = Ok(None);
                    rh.set_window_title(
                        &thread,
                        &format!(
                            "t={} score={} result={}",
                            sim.time(),
                            sim.score(),
                            match current_sim_result {
                                Ok(Some(v)) => format!("{}", v),
                                Ok(None) => "<running>".to_string(),
                                Err(pos) => format!("<error at {:?}>", pos),
                            }
                        ),
                    );
                }
                KeyboardKey::KEY_D => {
                    current_sim_result = sim.step();
                    rh.set_window_title(
                        &thread,
                        &format!(
                            "t={} score={} result={}",
                            sim.time(),
                            sim.score(),
                            match current_sim_result {
                                Ok(Some(v)) => format!("{}", v),
                                Ok(None) => "<running>".to_string(),
                                Err(pos) => format!("<error at {:?}>", pos),
                            }
                        ),
                    );
                }
                _ => {}
            }
        }

        if need_to_sleep {
            thread::sleep(time::Duration::from_millis(5));
        }
    }
}

fn render_sim(d: &mut RaylibDrawHandle, state: &GuiState, sim: &ThreeDSimulator) {
    const CELL_SIZE: i32 = 30;

    for (pos, cell) in sim.cells() {
        d.draw_rectangle(
            state.viewport_offset.x + pos.x * CELL_SIZE,
            state.viewport_offset.y + pos.y * CELL_SIZE,
            CELL_SIZE,
            CELL_SIZE,
            colors::SOLARIZED_BASE02,
        );
        match cell {
            Cell::Data(v) => {
                d.draw_text(
                    &format!("{}", v),
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    13,
                    colors::SOLARIZED_BASE0,
                );
            }
            Cell::MoveLeft => {
                d.draw_text(
                    "<",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_ORANGE,
                );
            }
            Cell::MoveRight => {
                d.draw_text(
                    ">",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_ORANGE,
                );
            }
            Cell::MoveUp => {
                d.draw_text(
                    "^",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_ORANGE,
                );
            }
            Cell::MoveDown => {
                d.draw_text(
                    "v",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_ORANGE,
                );
            }
            Cell::Add => {
                d.draw_text(
                    "+",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_VIOLET,
                );
            }
            Cell::Subtract => {
                d.draw_text(
                    "-",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_VIOLET,
                );
            }
            Cell::Multiply => {
                d.draw_text(
                    "*",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_VIOLET,
                );
            }
            Cell::Divide => {
                d.draw_text(
                    "/",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_VIOLET,
                );
            }
            Cell::Modulo => {
                d.draw_text(
                    "%",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_VIOLET,
                );
            }
            Cell::Equal => {
                d.draw_text(
                    "=",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_RED,
                );
            }
            Cell::NotEqual => {
                d.draw_text(
                    "#",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_RED,
                );
            }
            Cell::TimeWarp => {
                d.draw_text(
                    "@",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_CYAN,
                );
            }
            Cell::Submit => {
                d.draw_text(
                    "S",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_BLUE,
                );
            }
        }
    }

    let start_x = state.viewport_offset.x % CELL_SIZE;
    let start_y = state.viewport_offset.y % CELL_SIZE;
    for x in (start_x..state.width).step_by(CELL_SIZE as usize) {
        d.draw_line(x, 0, x, state.height, colors::SOLARIZED_BASE02);
    }
    for y in (start_y..state.height).step_by(CELL_SIZE as usize) {
        d.draw_line(0, y, state.width, y, colors::SOLARIZED_BASE02);
    }
}
