use std::{collections::HashMap, path::PathBuf, thread, time};

use crate::{
    geometry::Vector2D,
    three_d::sim::{SimulationStepResult, ThreeDSimulator},
};

use super::{board::ThreeDBoard, sim::Cell};

use raylib::prelude::*;

const CELL_SIZE: i32 = 30;

#[derive(Debug, Default)]
struct GuiState {
    width: i32,
    height: i32,
    viewport_offset: Vector2D,
    viewport_drag_point: Option<Vector2>,
    mouse_pos: Vector2,
    drag_start: Option<Vector2>,
    drag_group: Option<Vec<Vector2D>>,
    selected_pos: Vector2D,
    selection_rect: Option<(Vector2, Vector2)>,
    selection_group: Option<Vec<Vector2D>>,
    edit_mode: bool,
    edited_value: String,
    history: Vec<ThreeDSimulator>,
}

impl GuiState {
    fn screen_to_sim_coords(&self, pos: Vector2) -> Vector2D {
        let mut x = pos.x as i32 - self.viewport_offset.x;
        let mut y = pos.y as i32 - self.viewport_offset.y;
        if x < 0 {
            x -= CELL_SIZE;
        }
        if y < 0 {
            y -= CELL_SIZE;
        }
        Vector2D::new(x / CELL_SIZE, y / CELL_SIZE)
    }
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

pub fn gui_main(mut filepath: Option<PathBuf>, a: i64, b: i64) {
    const WINDOW_WIDTH: i32 = 1024;
    const WINDOW_HEIGHT: i32 = 768;

    let board = if let Some(path) = filepath.as_ref() {
        let board_file = std::fs::read_to_string(path).expect("Failed to read the board file");
        ThreeDBoard::load(&board_file)
    } else {
        ThreeDBoard::default()
    };

    let mut state = GuiState {
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        ..Default::default()
    };
    let mut sim = ThreeDSimulator::new(board, a, b);

    center_viewport(&sim, &mut state);

    #[allow(unused_assignments)]
    let mut current_sim_result = SimulationStepResult::Ok;

    let (mut rh, thread) = raylib::init().size(WINDOW_WIDTH, WINDOW_HEIGHT).build();

    rh.set_exit_key(None);

    update_window_title(&rh, &thread, &sim, current_sim_result, filepath.as_ref());

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
  Left button + Shift: select multiple cells
  Right button: drag the viewport
  Drag with left button: move selected cells
  Ctrl + drag with left button: copy selected cells
  Drag with right button: move the viewport

File management:
  Ctrl+N: new board
  Ctrl+O: open a board
  Ctrl+R: reload the current board
  Ctrl+S: save the board
  Ctrl+Shift+S: save the board as

Simulation:
  Q: undo (revert to the previous state, undoes time travel too)
  Shift+Q: step back in simulation history (time travel)
  E: execute one step of the simulation
  Shift+E: make the current simulation state the new initial state
  W: run the simulation (small delay between steps for visual feedback)
  Shift+W: run the simulation without delay

Misc:
  C: center the viewport
  ?: show this help
"#;

                d.draw_text(HELP_TEXT, 10, 10, 18, colors::SOLARIZED_BASE0);
            } else {
                render_sim(&mut d, &state, &sim);
            }
        }

        let mouse_pos = rh.get_mouse_position();
        state.mouse_pos = mouse_pos;
        if state.selection_rect.is_some() {
            state.selection_rect = Some((state.selection_rect.unwrap().0, mouse_pos));
        }

        if rh.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            if let Some(start) = state.drag_start {
                if (mouse_pos - start).length() > 5.0 {
                    state.drag_group = Some(
                        state
                            .selection_group
                            .clone()
                            .unwrap_or_else(|| vec![state.selected_pos]),
                    );
                }
            }
        }

        if rh.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            state.selected_pos = state.screen_to_sim_coords(mouse_pos);
            if let Some(group) = state.selection_group.as_ref() {
                if !group.contains(&state.selected_pos) {
                    state.selection_group = None;
                }
            }
            if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                state.selection_rect = Some((mouse_pos, mouse_pos));
            } else if sim.cells().contains_key(&state.selected_pos) {
                state.drag_start = Some(mouse_pos);
            }
        } else if rh.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            state.viewport_drag_point = Some(mouse_pos);
        }

        if rh.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            if let Some(group) = state.drag_group.take() {
                let drag_start_pos = state.screen_to_sim_coords(state.drag_start.unwrap());
                let mut cells = HashMap::new();
                if !rh.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
                    for pos in &group {
                        cells.insert(*pos, sim.remove_cell(*pos).unwrap());
                    }
                }
                let mut new_selection_group = vec![];
                for pos in &group {
                    let new_pos =
                        state.screen_to_sim_coords(state.mouse_pos) - drag_start_pos + *pos;
                    let cell = cells
                        .remove(pos)
                        .unwrap_or_else(|| *sim.cells().get(pos).unwrap());
                    sim.set_cell(new_pos, cell);
                    new_selection_group.push(new_pos);
                    if state.selected_pos == *pos {
                        state.selected_pos = new_pos;
                    }
                }
                state.selection_group = Some(new_selection_group);
            } else if let Some((start, end)) = state.selection_rect.take() {
                let start = state.screen_to_sim_coords(start);
                let end = state.screen_to_sim_coords(end);
                let mut selected_cells = Vec::new();
                for x in start.x.min(end.x)..=start.x.max(end.x) {
                    for y in start.y.min(end.y)..=start.y.max(end.y) {
                        let pos = Vector2D::new(x, y);
                        if sim.cells().contains_key(&pos) {
                            selected_cells.push(pos);
                        }
                    }
                }
                state.selection_group = Some(selected_cells);
            }
            state.drag_start = None;
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
            if result != SimulationStepResult::AlreadyFinished {
                current_sim_result = result;
                update_window_title(&rh, &thread, &sim, current_sim_result, filepath.as_ref());
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
                            current_sim_result = result;
                        }
                    } else if let Some(prev_sim) = state.history.pop() {
                        sim = prev_sim;
                        current_sim_result = SimulationStepResult::Ok;
                    }
                    update_window_title(&rh, &thread, &sim, current_sim_result, filepath.as_ref());
                }
                KeyboardKey::KEY_E => {
                    if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || rh.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT)
                    {
                        sim = ThreeDSimulator::new(sim.as_board().clone(), a, b);
                        current_sim_result = SimulationStepResult::Ok;
                        update_window_title(
                            &rh,
                            &thread,
                            &sim,
                            current_sim_result,
                            filepath.as_ref(),
                        );
                    } else {
                        state.history.push(sim.clone());
                        let result = sim.step();
                        match result {
                            SimulationStepResult::AlreadyFinished => {
                                state.history.pop();
                            }
                            _ => {
                                current_sim_result = result;
                                update_window_title(
                                    &rh,
                                    &thread,
                                    &sim,
                                    current_sim_result,
                                    filepath.as_ref(),
                                );
                            }
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
                        filepath = Some(path);
                        let board_file = std::fs::read_to_string(filepath.as_ref().unwrap())
                            .expect("Failed to read the board file");
                        let board = ThreeDBoard::load(&board_file);
                        sim = ThreeDSimulator::new(board, a, b);
                        current_sim_result = SimulationStepResult::Ok;
                        state.history.clear();
                        center_viewport(&sim, &mut state);
                        update_window_title(
                            &rh,
                            &thread,
                            &sim,
                            current_sim_result,
                            filepath.as_ref(),
                        );
                    }
                }
                KeyboardKey::KEY_R if rh.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) => {
                    if let Some(path) = filepath.as_ref() {
                        let board_file =
                            std::fs::read_to_string(path).expect("Failed to read the board file");
                        let board = ThreeDBoard::load(&board_file);
                        sim = ThreeDSimulator::new(board, a, b);
                        current_sim_result = SimulationStepResult::Ok;
                        state.history.clear();
                        update_window_title(&rh, &thread, &sim, current_sim_result, Some(path));
                    }
                }
                KeyboardKey::KEY_N if rh.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) => {
                    sim = ThreeDSimulator::new(ThreeDBoard::default(), a, b);
                    current_sim_result = SimulationStepResult::Ok;
                    state.history.clear();
                    filepath = None;
                    update_window_title(&rh, &thread, &sim, current_sim_result, filepath.as_ref());
                }
                KeyboardKey::KEY_S if rh.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) => {
                    if rh.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || filepath.is_none() {
                        let cwd = std::env::current_dir().expect("Failed to get current directory");
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(cwd)
                            .set_title("Save board")
                            .save_file()
                        {
                            filepath = Some(path);
                            let board = sim.as_board().save();
                            std::fs::write(filepath.as_ref().unwrap(), board)
                                .expect("Failed to write to file");
                        }
                    } else {
                        let board = sim.as_board().save();
                        std::fs::write(filepath.as_ref().unwrap(), board)
                            .expect("Failed to write to file");
                    }
                }
                KeyboardKey::KEY_C => {
                    center_viewport(&sim, &mut state);
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
                                        filepath.as_ref(),
                                    );
                                }
                                Some(Cell::InputB) => {
                                    sim.set_b(v);
                                    update_window_title(
                                        &rh,
                                        &thread,
                                        &sim,
                                        current_sim_result,
                                        filepath.as_ref(),
                                    );
                                }
                                _ => sim.set_cell(state.selected_pos, Cell::Data(v)),
                            }
                        }
                        state.edited_value.clear();
                    }
                }
                KeyboardKey::KEY_DELETE => {
                    if let Some(group) = state.selection_group.take() {
                        for pos in group {
                            sim.remove_cell(pos);
                        }
                    } else {
                        sim.remove_cell(state.selected_pos);
                    }
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

fn center_viewport(sim: &ThreeDSimulator, state: &mut GuiState) {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    for pos in sim.cells().keys() {
        min_x = min_x.min(pos.x);
        min_y = min_y.min(pos.y);
        max_x = max_x.max(pos.x);
        max_y = max_y.max(pos.y);
    }
    let board_width = (max_x - min_x + 1) * 30;
    let board_height = (max_y - min_y + 1) * 30;
    state.viewport_offset = Vector2D::new(
        (state.width - board_width) / 2 - 30,
        (state.height - board_height) / 2 - 30,
    );
}

fn update_window_title(
    rh: &RaylibHandle,
    thread: &RaylibThread,
    sim: &ThreeDSimulator,
    current_sim_result: SimulationStepResult,
    path: Option<&PathBuf>,
) {
    rh.set_window_title(
        thread,
        &format!(
            "[{}] a={} b={} t={} step={} score={} result={}",
            if let Some(path) = path {
                path.file_name().unwrap().to_string_lossy().into_owned()
            } else {
                "untitled".to_string()
            },
            sim.a(),
            sim.b(),
            sim.time(),
            sim.steps_taken(),
            sim.score(),
            match current_sim_result {
                SimulationStepResult::Finished(v) => format!("{}", v),
                SimulationStepResult::Ok => "<running>".to_string(),
                SimulationStepResult::AlreadyFinished => unreachable!("Must be handled elsewhere"),
                SimulationStepResult::Error(pos) => format!("<error at {:?}>", pos),
            }
        ),
    );
}

fn render_sim(d: &mut RaylibDrawHandle, state: &GuiState, sim: &ThreeDSimulator) {
    for (pos, cell) in sim.cells() {
        if state.drag_group.as_ref().map_or(true, |g| !g.contains(pos))
            || d.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
        {
            draw_cell(d, state, pos, cell, Vector2::zero());
        }
    }

    if let Some(selected_cells) = state.selection_group.as_ref() {
        for pos in selected_cells {
            d.draw_rectangle_lines(
                state.viewport_offset.x + pos.x * CELL_SIZE,
                state.viewport_offset.y + pos.y * CELL_SIZE + 1,
                CELL_SIZE - 1,
                CELL_SIZE - 1,
                colors::SOLARIZED_BASE01,
            );
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
                colors::SOLARIZED_BASE1
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

    if let Some((start, end)) = state.selection_rect {
        let min_x = start.x.min(end.x);
        let min_y = start.y.min(end.y);
        let max_x = start.x.max(end.x);
        let max_y = start.y.max(end.y);
        d.draw_rectangle_lines(
            min_x as i32,
            min_y as i32,
            (max_x - min_x) as i32,
            (max_y - min_y) as i32,
            colors::SOLARIZED_BASE01,
        );
    }

    if let Some(group) = state.drag_group.as_ref() {
        let drag_offset = state.mouse_pos - state.drag_start.unwrap();
        let drag_start_pos = state.screen_to_sim_coords(state.drag_start.unwrap());
        for pos in group {
            let new_pos = state.screen_to_sim_coords(state.mouse_pos) - drag_start_pos + *pos;
            d.draw_rectangle(
                state.viewport_offset.x + new_pos.x * CELL_SIZE,
                state.viewport_offset.y + new_pos.y * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE,
                colors::SOLARIZED_BASE02,
            );
        }
        for pos in group {
            if let Some(cell) = sim.cells().get(pos) {
                draw_cell(d, state, pos, cell, drag_offset);
            }
            d.draw_rectangle_lines(
                state.viewport_offset.x + (drag_offset.x as i32) + pos.x * CELL_SIZE,
                state.viewport_offset.y + (drag_offset.y as i32) + pos.y * CELL_SIZE + 1,
                CELL_SIZE - 1,
                CELL_SIZE - 1,
                colors::SOLARIZED_BASE01,
            );
        }
    }
}

fn draw_cell(
    d: &mut RaylibDrawHandle,
    state: &GuiState,
    pos: &Vector2D,
    cell: &Cell,
    offset: Vector2,
) {
    let x = state.viewport_offset.x + pos.x * CELL_SIZE + (offset.x as i32);
    let y = state.viewport_offset.y + pos.y * CELL_SIZE + (offset.y as i32);

    d.draw_rectangle(x, y, CELL_SIZE, CELL_SIZE, colors::SOLARIZED_BASE02);
    match cell {
        Cell::Data(v) => {
            d.draw_text(&format!("{}", v), x + 5, y + 5, 13, colors::SOLARIZED_BASE0);
        }
        Cell::MoveLeft => {
            d.draw_text("<", x + 5, y + 5, 25, colors::SOLARIZED_ORANGE);
        }
        Cell::MoveRight => {
            d.draw_text(">", x + 5, y + 5, 25, colors::SOLARIZED_ORANGE);
        }
        Cell::MoveUp => {
            d.draw_text("^", x + 5, y + 5, 25, colors::SOLARIZED_ORANGE);
        }
        Cell::MoveDown => {
            d.draw_text("v", x + 5, y + 5, 25, colors::SOLARIZED_ORANGE);
        }
        Cell::Add => {
            d.draw_text("+", x + 5, y + 5, 25, colors::SOLARIZED_VIOLET);
        }
        Cell::Subtract => {
            d.draw_text("-", x + 5, y + 5, 25, colors::SOLARIZED_VIOLET);
        }
        Cell::Multiply => {
            d.draw_text("*", x + 5, y + 5, 25, colors::SOLARIZED_VIOLET);
        }
        Cell::Divide => {
            d.draw_text("/", x + 5, y + 5, 25, colors::SOLARIZED_VIOLET);
        }
        Cell::Modulo => {
            d.draw_text("%", x + 5, y + 5, 25, colors::SOLARIZED_VIOLET);
        }
        Cell::Equal => {
            d.draw_text("=", x + 5, y + 5, 25, colors::SOLARIZED_RED);
        }
        Cell::NotEqual => {
            d.draw_text("#", x + 5, y + 5, 25, colors::SOLARIZED_RED);
        }
        Cell::TimeWarp => {
            d.draw_text("@", x + 5, y + 5, 25, colors::SOLARIZED_CYAN);
        }
        Cell::Submit => {
            d.draw_text("S", x + 5, y + 5, 25, colors::SOLARIZED_BLUE);
        }
        Cell::InputA => {
            d.draw_text("A", x + 5, y + 5, 25, colors::SOLARIZED_GREEN);
        }
        Cell::InputB => {
            d.draw_text("B", x + 5, y + 5, 25, colors::SOLARIZED_GREEN);
        }
    }
}
