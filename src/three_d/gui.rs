use std::{
    path::{Path, PathBuf},
    thread, time,
};

use crate::{
    geometry::Vector2D,
    three_d::sim::{SimulationStepResult, ThreeDSimulator},
};

use super::{board::ThreeDBoard, sim::Cell};

use raylib::prelude::*;

#[derive(Debug, Default)]
struct GuiState {
    width: i32,
    height: i32,
    viewport_offset: Vector2D,
    viewport_drag_point: Option<Vector2>,
    selected_pos: Vector2D,
    edit_mode: bool,
    edited_value: String,
    history: Vec<ThreeDSimulator>,
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

pub fn gui_main(mut filepath: PathBuf, a: i64, b: i64) {
    const WINDOW_WIDTH: i32 = 1024;
    const WINDOW_HEIGHT: i32 = 768;

    let board_file = std::fs::read_to_string(&filepath).expect("Failed to read the board file");
    let board = ThreeDBoard::load(&board_file);

    let mut state = GuiState {
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        ..Default::default()
    };
    let mut sim = ThreeDSimulator::new(board, a, b);

    #[allow(unused_assignments)]
    let mut current_sim_result = Ok(SimulationStepResult::Ok);

    let (mut rh, thread) = raylib::init().size(WINDOW_WIDTH, WINDOW_HEIGHT).build();

    update_window_title(&rh, &thread, &sim, current_sim_result, &filepath);

    while !rh.window_should_close() {
        {
            let mut d = rh.begin_drawing(&thread);
            d.clear_background(colors::SOLARIZED_BASE03);
            if d.is_key_down(KeyboardKey::KEY_SLASH)
                && (d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                    || d.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT))
            {
                const HELP_TEXT: &str = r#"===== HELP =====

Arrow keys: move the selection

Enter: toggle edit mode
 - In edit mode, type a number and press Enter to set the selected cell to that number
 - If over A or B inputs, will set the input value instead
 - Only numbers, minus sign and backspace are allowed

 Cell keys:
  < > ^ v: set the selected cell to move left/right/up/down
  - + - * / %: set the selected cell to add/subtract/multiply/divide/modulo
  = #: set the selected cell to equal/not equal
  @: set the selected cell to time warp
  S: set the selected cell to submit
  A B: set the selected cell to input A or B
  Delete: remove the selected cell

Mouse actions:
  Left button: select a cell
  Right button: drag the viewport

File management:
  Ctrl+O: open a board
  Ctrl+R: reload the current board
  Ctrl+S: save the board
  Ctrl+Shift+S: save the board as

Simulation:
  Q: undo (revert to the previous state, undoes time travel too)
  Shift+Q: step back in simulation history (time travel)
  E: execute one step of the simulation
  W: run the simulation (small delay between steps for visual feedback)
  Shift+W: run the simulation without delay

ESC: close the program
"#;

                d.draw_text(HELP_TEXT, 10, 10, 18, colors::SOLARIZED_BASE0);
            } else {
                render_sim(&mut d, &state, &sim);
            }
        }

        let mouse_pos = rh.get_mouse_position();

        if rh.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            // TODO: cell size
            state.selected_pos = Vector2D::new(
                (mouse_pos.x as i32 - state.viewport_offset.x) / 30,
                (mouse_pos.y as i32 - state.viewport_offset.y) / 30,
            );
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

        let mut delay_ms = 5;
        if rh.is_key_down(KeyboardKey::KEY_W) {
            let result = sim.step();
            if result != Ok(SimulationStepResult::AlreadyFinished) {
                current_sim_result = result;
                update_window_title(&rh, &thread, &sim, current_sim_result, &filepath);
                if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                    || rh.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT)
                {
                    delay_ms = 0;
                } else {
                    delay_ms = 50;
                }
            }
        } else if let Some(key) = rh.get_key_pressed() {
            match key {
                KeyboardKey::KEY_Q => {
                    if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || rh.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT)
                    {
                        let result = sim.step_back();
                        if result != SimulationStepResult::AlreadyFinished {
                            state.history.push(sim.clone());
                            current_sim_result = Ok(result);
                        }
                    } else if let Some(prev_sim) = state.history.pop() {
                        sim = prev_sim;
                        current_sim_result = Ok(SimulationStepResult::Ok);
                    }
                    update_window_title(&rh, &thread, &sim, current_sim_result, &filepath);
                }
                KeyboardKey::KEY_E => {
                    state.history.push(sim.clone());
                    let result = sim.step();
                    match result {
                        Ok(SimulationStepResult::AlreadyFinished) => {
                            state.history.pop();
                        }
                        _ => {
                            current_sim_result = result;
                            update_window_title(&rh, &thread, &sim, current_sim_result, &filepath);
                        }
                    }
                }
                KeyboardKey::KEY_O if rh.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) => {
                    let cwd = std::env::current_dir().expect("Failed to get current directory");
                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory(cwd)
                        .set_title("Open board")
                        .pick_file()
                    {
                        filepath = path;
                        let board_file = std::fs::read_to_string(&filepath)
                            .expect("Failed to read the board file");
                        let board = ThreeDBoard::load(&board_file);
                        sim = ThreeDSimulator::new(board, a, b);
                        current_sim_result = Ok(SimulationStepResult::Ok);
                        state.history.clear();
                        update_window_title(&rh, &thread, &sim, current_sim_result, &filepath);
                    }
                }
                KeyboardKey::KEY_R if rh.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) => {
                    let board_file =
                        std::fs::read_to_string(&filepath).expect("Failed to read the board file");
                    let board = ThreeDBoard::load(&board_file);
                    sim = ThreeDSimulator::new(board, a, b);
                    current_sim_result = Ok(SimulationStepResult::Ok);
                    state.history.clear();
                    update_window_title(&rh, &thread, &sim, current_sim_result, &filepath);
                }
                KeyboardKey::KEY_S if rh.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) => {
                    if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                        let cwd = std::env::current_dir().expect("Failed to get current directory");
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(cwd)
                            .set_title("Save board")
                            .save_file()
                        {
                            filepath = path;
                            let board = sim.as_board().save();
                            std::fs::write(&filepath, board).expect("Failed to write to file");
                        }
                    } else {
                        let board = sim.as_board().save();
                        std::fs::write(&filepath, board).expect("Failed to write to file");
                    }
                }
                KeyboardKey::KEY_LEFT => {
                    state.selected_pos = state.selected_pos.left();
                }
                KeyboardKey::KEY_RIGHT => {
                    state.selected_pos = state.selected_pos.right();
                }
                KeyboardKey::KEY_UP => {
                    state.selected_pos = state.selected_pos.up();
                }
                KeyboardKey::KEY_DOWN => {
                    state.selected_pos = state.selected_pos.down();
                }
                KeyboardKey::KEY_ENTER => {
                    state.edit_mode = !state.edit_mode;
                    if !state.edit_mode {
                        if let Ok(v) = state.edited_value.parse::<i64>() {
                            match sim.cells().get(&state.selected_pos) {
                                Some(Cell::InputA) => {
                                    sim.set_a(v);
                                    update_window_title(
                                        &rh,
                                        &thread,
                                        &sim,
                                        current_sim_result,
                                        &filepath,
                                    );
                                }
                                Some(Cell::InputB) => {
                                    sim.set_b(v);
                                    update_window_title(
                                        &rh,
                                        &thread,
                                        &sim,
                                        current_sim_result,
                                        &filepath,
                                    );
                                }
                                _ => sim.set_cell(state.selected_pos, Cell::Data(v)),
                            }
                        }
                        state.edited_value.clear();
                    }
                }
                KeyboardKey::KEY_DELETE => {
                    sim.remove_cell(state.selected_pos);
                }
                // <
                KeyboardKey::KEY_COMMA
                    if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || rh.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) =>
                {
                    sim.set_cell(state.selected_pos, Cell::MoveLeft);
                }
                // >
                KeyboardKey::KEY_PERIOD
                    if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || rh.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) =>
                {
                    sim.set_cell(state.selected_pos, Cell::MoveRight);
                }
                // ^
                KeyboardKey::KEY_SIX
                    if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || rh.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) =>
                {
                    sim.set_cell(state.selected_pos, Cell::MoveUp);
                }
                // v
                KeyboardKey::KEY_V => {
                    sim.set_cell(state.selected_pos, Cell::MoveDown);
                }
                // +
                KeyboardKey::KEY_EQUAL
                    if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || rh.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) =>
                {
                    sim.set_cell(state.selected_pos, Cell::Add);
                }
                // -
                KeyboardKey::KEY_MINUS if !state.edit_mode => {
                    sim.set_cell(state.selected_pos, Cell::Subtract);
                }
                // *
                KeyboardKey::KEY_EIGHT if !state.edit_mode => {
                    sim.set_cell(state.selected_pos, Cell::Multiply);
                }
                // /
                KeyboardKey::KEY_SLASH
                    if !(rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || rh.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT)) =>
                {
                    sim.set_cell(state.selected_pos, Cell::Divide);
                }
                // %
                KeyboardKey::KEY_FIVE
                    if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || rh.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) =>
                {
                    sim.set_cell(state.selected_pos, Cell::Modulo);
                }
                // =
                KeyboardKey::KEY_EQUAL => {
                    sim.set_cell(state.selected_pos, Cell::Equal);
                }
                // #
                KeyboardKey::KEY_THREE
                    if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || rh.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) =>
                {
                    sim.set_cell(state.selected_pos, Cell::NotEqual);
                }
                // @
                KeyboardKey::KEY_TWO
                    if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || rh.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) =>
                {
                    sim.set_cell(state.selected_pos, Cell::TimeWarp);
                }
                // S
                KeyboardKey::KEY_S => {
                    sim.set_cell(state.selected_pos, Cell::Submit);
                }
                // A
                KeyboardKey::KEY_A => {
                    sim.set_cell(state.selected_pos, Cell::InputA);
                }
                // B
                KeyboardKey::KEY_B => {
                    sim.set_cell(state.selected_pos, Cell::InputB);
                }
                // Numbers
                KeyboardKey::KEY_ZERO if state.edit_mode => {
                    state.edited_value += "0";
                }
                KeyboardKey::KEY_ONE if state.edit_mode => {
                    state.edited_value += "1";
                }
                KeyboardKey::KEY_TWO if state.edit_mode => {
                    state.edited_value += "2";
                }
                KeyboardKey::KEY_THREE if state.edit_mode => {
                    state.edited_value += "3";
                }
                KeyboardKey::KEY_FOUR if state.edit_mode => {
                    state.edited_value += "4";
                }
                KeyboardKey::KEY_FIVE if state.edit_mode => {
                    state.edited_value += "5";
                }
                KeyboardKey::KEY_SIX if state.edit_mode => {
                    state.edited_value += "6";
                }
                KeyboardKey::KEY_SEVEN if state.edit_mode => {
                    state.edited_value += "7";
                }
                KeyboardKey::KEY_EIGHT if state.edit_mode => {
                    state.edited_value += "8";
                }
                KeyboardKey::KEY_NINE if state.edit_mode => {
                    state.edited_value += "9";
                }
                KeyboardKey::KEY_MINUS if state.edit_mode => {
                    state.edited_value += "-";
                }
                KeyboardKey::KEY_BACKSPACE if state.edit_mode => {
                    state.edited_value.pop();
                }
                _ => {}
            }
        }

        if delay_ms > 0 {
            thread::sleep(time::Duration::from_millis(delay_ms));
        }
    }
}

fn update_window_title(
    rh: &RaylibHandle,
    thread: &RaylibThread,
    sim: &ThreeDSimulator,
    current_sim_result: Result<SimulationStepResult, Vector2D>,
    path: &Path,
) {
    rh.set_window_title(
        thread,
        &format!(
            "[{}] a={} b={} t={} step={} score={} result={}",
            path.file_name().unwrap().to_string_lossy(),
            sim.a(),
            sim.b(),
            sim.time(),
            sim.steps_taken(),
            sim.score(),
            match current_sim_result {
                Ok(SimulationStepResult::Finished(v)) => format!("{}", v),
                Ok(SimulationStepResult::Ok) => "<running>".to_string(),
                Ok(SimulationStepResult::AlreadyFinished) =>
                    unreachable!("Must be handled elsewhere"),
                Err(pos) => format!("<error at {:?}>", pos),
            }
        ),
    );
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
            Cell::InputA => {
                d.draw_text(
                    "A",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_GREEN,
                );
            }
            Cell::InputB => {
                d.draw_text(
                    "B",
                    state.viewport_offset.x + pos.x * CELL_SIZE + 5,
                    state.viewport_offset.y + pos.y * CELL_SIZE + 5,
                    25,
                    colors::SOLARIZED_GREEN,
                );
            }
        }
    }

    {
        d.draw_rectangle_lines(
            state.viewport_offset.x + state.selected_pos.x * CELL_SIZE,
            state.viewport_offset.y + state.selected_pos.y * CELL_SIZE + 1,
            CELL_SIZE - 1,
            CELL_SIZE - 1,
            if state.edit_mode {
                colors::SOLARIZED_RED
            } else {
                colors::SOLARIZED_BASE01
            },
        );
        if state.edit_mode {
            d.draw_text(
                &state.edited_value,
                state.viewport_offset.x + state.selected_pos.x * CELL_SIZE + 5,
                state.viewport_offset.y + state.selected_pos.y * CELL_SIZE + 5,
                25,
                colors::SOLARIZED_RED,
            );
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
