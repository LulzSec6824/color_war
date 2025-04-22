/// A chain reaction game where players compete to control the board by strategically placing and exploding tiles.
/// Players take turns placing tiles, which can chain react and capture neighboring cells.
/// The game continues until only one player remains on the board.
use ggez::{
    Context, ContextBuilder, GameResult,
    event::{self, EventHandler},
    graphics::{self, Drawable as _},
    input::mouse::MouseButton,
};
use rand::seq::SliceRandom;
use std::collections::VecDeque;
use std::time::Instant;

const ROWS: usize = 8;
const COLS: usize = 8;
const CELL_SIZE: f32 = 50.0;
const PLAYERS: usize = 4;
const ANIMATION_DURATION: f32 = 0.1;
const BOARD_MARGIN: f32 = 4.0 * CELL_SIZE;

/// Represents a single cell on the game board.
/// Each cell can be owned by a player and contains a power level that determines when it explodes.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Cell {
    owner: Option<usize>,
    power: u8,
    animation_start: Option<Instant>,
    animation_from: Option<(f32, f32)>,
    is_exploding: bool,
    animation_delay: f32,
}

impl Cell {
    /// Creates a new empty cell with no owner and zero power.
    ///
    /// # Returns
    /// * A new Cell instance initialized with default values
    fn new() -> Self {
        Self {
            owner: None,
            power: 0,
            animation_start: None,
            animation_from: None,
            is_exploding: false,
            animation_delay: 0.0,
        }
    }
}

/// The main game state that manages the game board, players, and game logic.
/// Handles player turns, tile placement, explosions, and win conditions.
struct GameState {
    grid: [[Cell; COLS]; ROWS],
    players: usize,
    current_player: usize,
    animations: Vec<(usize, usize, (f32, f32))>,
    player_order: Vec<usize>,
    turn_number: usize,
    turn_messages: Vec<String>,
    first_moves: Vec<bool>,
    players_alive: Vec<bool>,
    game_over: bool,
    winner: Option<usize>,
}

impl GameState {
    /// Creates a new game state with initialized board and randomized player order.
    ///
    /// # Returns
    /// * A new GameState instance with empty board and initial game settings
    fn new() -> Self {
        let mut rng = rand::rng();
        let mut player_order: Vec<usize> = (0..PLAYERS).collect();
        player_order.shuffle(&mut rng);

        let turn_messages = vec![
            "Red player's turn - Place your tile!".to_string(),
            "Green player's turn - Place your tile!".to_string(),
            "Blue player's turn - Place your tile!".to_string(),
            "Yellow player's turn - Place your tile!".to_string(),
        ];

        Self {
            grid: [[Cell::new(); COLS]; ROWS],
            players: PLAYERS,
            current_player: player_order[0],
            animations: Vec::new(),
            player_order,
            turn_number: 0,
            turn_messages,
            first_moves: vec![true; PLAYERS],
            players_alive: vec![true; PLAYERS],
            game_over: false,
            winner: None,
        }
    }

    /// Returns the color associated with a given player number.
    ///
    /// # Arguments
    /// * `player` - The player number (0-3)
    ///
    /// # Returns
    /// * The graphics::Color associated with the player
    fn get_player_color(&self, player: usize) -> graphics::Color {
        match player {
            0 => graphics::Color::from_rgb(255, 0, 0),   // Red
            1 => graphics::Color::from_rgb(0, 255, 0),   // Green
            2 => graphics::Color::from_rgb(0, 0, 255),   // Blue
            3 => graphics::Color::from_rgb(255, 255, 0), // Yellow
            _ => graphics::Color::WHITE,
        }
    }

    #[allow(dead_code)]
    /// Returns the name associated with a given player number.
    ///
    /// # Arguments
    /// * `player` - The player number (0-3)
    ///
    /// # Returns
    /// * The string name of the player ("Red", "Green", etc.)
    fn get_player_name(&self, player: usize) -> String {
        match player {
            0 => "Red".to_string(),
            1 => "Green".to_string(),
            2 => "Blue".to_string(),
            3 => "Yellow".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    /// Returns a vector of valid neighboring cell coordinates for a given position.
    ///
    /// # Arguments
    /// * `row` - The row index of the cell
    /// * `col` - The column index of the cell
    ///
    /// # Returns
    /// * Vector of (row, col) tuples representing valid neighboring positions
    fn get_neighbors(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        if row > 0 {
            neighbors.push((row - 1, col));
        }
        if row < ROWS - 1 {
            neighbors.push((row + 1, col));
        }
        if col > 0 {
            neighbors.push((row, col - 1));
        }
        if col < COLS - 1 {
            neighbors.push((row, col + 1));
        }
        neighbors
    }

    /// Returns the maximum power capacity for a cell before it explodes.
    ///
    /// # Arguments
    /// * `_row` - The row index of the cell (unused)
    /// * `_col` - The column index of the cell (unused)
    ///
    /// # Returns
    /// * Fixed capacity value of 4 for all cells
    fn max_capacity(&self, _row: usize, _col: usize) -> u8 {
        4 // Fixed capacity of 4 for all cells
    }

    /// Checks if any players have been eliminated by having no tiles on the board.
    /// Updates player status and checks for game over condition.
    fn check_elimination(&mut self) {
        // Only check for elimination if all players have made their first move
        if self.first_moves.iter().any(|&first_move| first_move) {
            return;
        }

        for player in 0..self.players {
            let mut player_has_tiles = false;
            for row in 0..ROWS {
                for col in 0..COLS {
                    if self.grid[row][col].owner == Some(player) {
                        player_has_tiles = true;
                        break;
                    }
                }
                if player_has_tiles {
                    break;
                }
            }
            self.players_alive[player] = player_has_tiles;
        }

        let alive_count = self.players_alive.iter().filter(|&&alive| alive).count();
        if alive_count == 1 {
            self.winner = self.players_alive.iter().position(|&alive| alive);
            self.game_over = true;
        }
    }

    /// Attempts to place a tile for the current player at the specified position.
    /// Handles first move rules, tile placement, and triggers chain reactions.
    ///
    /// # Arguments
    /// * `row` - The row index to place the tile
    /// * `col` - The column index to place the tile
    fn place_tile(&mut self, row: usize, col: usize) {
        if self.game_over {
            return;
        }

        let cell = &mut self.grid[row][col];
        let is_first_move = self.first_moves[self.current_player];

        // First move: can only place in empty cells
        // Subsequent moves: must use existing circles of the current player
        if (is_first_move && cell.owner.is_none())
            || (!is_first_move && cell.owner == Some(self.current_player))
        {
            cell.owner = Some(self.current_player);
            if is_first_move {
                cell.power = 3;
                self.first_moves[self.current_player] = false;
            } else {
                cell.power += 1;
            }
            self.check_explosions();
            self.check_elimination();

            if !self.game_over {
                self.turn_number = (self.turn_number + 1) % self.players;
                self.current_player = self.player_order[self.turn_number];

                // Skip eliminated players
                while !self.players_alive[self.current_player] {
                    self.turn_number = (self.turn_number + 1) % self.players;
                    self.current_player = self.player_order[self.turn_number];
                }
            }
        }
    }

    /// Processes chain reactions when cells exceed their power capacity.
    /// Updates the board state and handles animations for exploding cells.
    fn check_explosions(&mut self) {
        let mut queue = VecDeque::new();
        let wave = 0;
        for r in 0..ROWS {
            for c in 0..COLS {
                if self.grid[r][c].power >= self.max_capacity(r, c) {
                    queue.push_back((r, c, wave));
                }
            }
        }
        while let Some((r, c, wave)) = queue.pop_front() {
            let owner = self.grid[r][c].owner;
            let source_x = c as f32 * CELL_SIZE + CELL_SIZE / 2.0 + BOARD_MARGIN;
            let source_y = r as f32 * CELL_SIZE + CELL_SIZE / 2.0 + BOARD_MARGIN;

            self.grid[r][c].power = 0;
            self.grid[r][c].owner = None;
            self.grid[r][c].is_exploding = true;
            self.grid[r][c].animation_start = Some(Instant::now());
            self.grid[r][c].animation_delay = wave as f32 * ANIMATION_DURATION;

            for (nr, nc) in self.get_neighbors(r, c) {
                let neighbor = &mut self.grid[nr][nc];
                let old_owner = neighbor.owner;
                neighbor.power += 1;
                neighbor.owner = owner;

                if old_owner != owner {
                    neighbor.animation_start = Some(Instant::now());
                    neighbor.animation_delay = wave as f32 * ANIMATION_DURATION;
                    neighbor.animation_from = Some((source_x, source_y));
                }

                if neighbor.power >= self.max_capacity(nr, nc) {
                    queue.push_back((nr, nc, wave + 1));
                }
            }
            self.animations.push((r, c, (source_x, source_y)));
        }
    }

    /// Calculates the progress of an animation based on elapsed time.
    ///
    /// # Arguments
    /// * `start` - The start time of the animation
    ///
    /// # Returns
    /// * A float between 0.0 and 1.0 representing animation progress
    fn get_animation_progress(&self, start: Instant, delay: f32) -> f32 {
        let elapsed = start.elapsed().as_secs_f32() - delay;
        if elapsed < 0.0 {
            return 0.0;
        }
        (elapsed / ANIMATION_DURATION).min(1.0)
    }

    /// Draws a single cell on the game board with its current state and animations.
    ///
    /// # Arguments
    /// * `canvas` - The graphics canvas to draw on
    /// * `ctx` - The game context
    /// * `row` - The row index of the cell
    /// * `col` - The column index of the cell
    ///
    /// # Returns
    /// * GameResult indicating success or failure
    fn draw_cell(
        &self,
        canvas: &mut graphics::Canvas,
        ctx: &mut Context,
        row: usize,
        col: usize,
    ) -> GameResult {
        let x = col as f32 * CELL_SIZE + BOARD_MARGIN;
        let y = row as f32 * CELL_SIZE + BOARD_MARGIN;

        let rect = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(x, y, CELL_SIZE - 2.0, CELL_SIZE - 2.0),
            graphics::Color::from_rgb(128, 128, 128),
        )?;
        canvas.draw(&rect, graphics::DrawParam::default());

        let border = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(2.0),
            graphics::Rect::new(x, y, CELL_SIZE - 2.0, CELL_SIZE - 2.0),
            graphics::Color::WHITE,
        )?;
        canvas.draw(&border, graphics::DrawParam::default());

        let cell = &self.grid[row][col];

        if let Some(player) = cell.owner {
            let mut circle_x = x + CELL_SIZE / 2.0;
            let mut circle_y = y + CELL_SIZE / 2.0;
            let mut scale = 1.0;
            let mut alpha = 1.0;

            if let Some(start) = cell.animation_start {
                let progress = self.get_animation_progress(start, cell.animation_delay);
                if progress < 1.0 {
                    if cell.is_exploding {
                        scale = 1.0 - progress * 0.5;
                        alpha = 1.0 - progress;
                    } else if let Some((from_x, from_y)) = cell.animation_from {
                        circle_x = from_x + (circle_x - from_x) * progress;
                        circle_y = from_y + (circle_y - from_y) * progress;
                        scale = 0.5 + progress * 0.5;
                        alpha = progress;
                    } else {
                        scale = 0.5 + progress * 0.5;
                    }
                }
            }

            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [circle_x, circle_y],
                CELL_SIZE / 3.0 * scale,
                0.1,
                {
                    let mut color = self.get_player_color(player);
                    color.a = alpha;
                    color
                },
            )?;
            canvas.draw(&circle, graphics::DrawParam::default());
        }

        if cell.power > 0 {
            let power_text = graphics::Text::new(cell.power.to_string());
            let text_dims = power_text.dimensions(ctx).unwrap();

            for offset_x in [-1.0, 0.0, 1.0].iter() {
                for offset_y in [-1.0, 0.0, 1.0].iter() {
                    canvas.draw(
                        &power_text,
                        graphics::DrawParam::default()
                            .color(graphics::Color::BLACK)
                            .dest([
                                x + (CELL_SIZE - text_dims.w) / 2.0 + offset_x,
                                y + (CELL_SIZE - text_dims.h) / 2.0 + offset_y,
                            ])
                            .scale([1.0, 1.0]),
                    );
                }
            }

            canvas.draw(
                &power_text,
                graphics::DrawParam::default()
                    .color(graphics::Color::WHITE)
                    .dest([
                        x + (CELL_SIZE - text_dims.w) / 2.0,
                        y + (CELL_SIZE - text_dims.h) / 2.0,
                    ])
                    .scale([1.0, 1.0]),
            );
        }

        Ok(())
    }

    /// Draws the current turn message or game over message on all sides of the board.
    ///
    /// # Arguments
    /// * `canvas` - The graphics canvas to draw on
    /// * `ctx` - The game context
    ///
    /// # Returns
    /// * GameResult indicating success or failure
    fn draw_turn_messages(&self, canvas: &mut graphics::Canvas, ctx: &mut Context) -> GameResult {
        if self.game_over {
            if let Some(winner) = self.winner {
                let winner_message = format!("{} player wins!", self.get_player_name(winner));
                let message_text = graphics::Text::new(winner_message);
                let text_dims = message_text.dimensions(ctx).unwrap();
                let x = (COLS as f32 * CELL_SIZE + 2.0 * BOARD_MARGIN - text_dims.w) / 2.0;
                let y = BOARD_MARGIN / 2.0 - text_dims.h / 2.0;

                canvas.draw(
                    &message_text,
                    graphics::DrawParam::default()
                        .color(self.get_player_color(winner))
                        .dest([x, y]),
                );
                return Ok(());
            }
        }

        let turn_message = &self.turn_messages[self.current_player];
        let message_text = graphics::Text::new(turn_message);
        let text_dims = message_text.dimensions(ctx).unwrap();

        let draw_outlined_text = |canvas: &mut graphics::Canvas, x: f32, y: f32, rotation: f32| {
            let outline_offsets = [
                (-1.0, -1.0),
                (0.0, -1.0),
                (1.0, -1.0),
                (-1.0, 0.0),
                (1.0, 0.0),
                (-1.0, 1.0),
                (0.0, 1.0),
                (1.0, 1.0),
            ];

            for (offset_x, offset_y) in outline_offsets.iter() {
                canvas.draw(
                    &message_text,
                    graphics::DrawParam::default()
                        .color(graphics::Color::BLACK)
                        .dest([x + offset_x, y + offset_y])
                        .rotation(rotation)
                        .scale([1.0, 1.0]),
                );
            }

            canvas.draw(
                &message_text,
                graphics::DrawParam::default()
                    .color(self.get_player_color(self.current_player))
                    .dest([x, y])
                    .rotation(rotation)
                    .scale([1.0, 1.0]),
            );
        };

        let top_y = BOARD_MARGIN / 2.0 - text_dims.h / 2.0;
        let bottom_y =
            BOARD_MARGIN + (ROWS as f32 * CELL_SIZE) + BOARD_MARGIN / 2.0 - text_dims.h / 2.0;
        let left_x = BOARD_MARGIN / 2.0 + text_dims.h / 2.0;
        let right_x =
            BOARD_MARGIN + (COLS as f32 * CELL_SIZE) + BOARD_MARGIN / 2.0 - text_dims.h / 2.0;

        draw_outlined_text(canvas, BOARD_MARGIN, top_y, 0.0);
        draw_outlined_text(canvas, BOARD_MARGIN, bottom_y, 0.0);
        draw_outlined_text(
            canvas,
            left_x,
            BOARD_MARGIN + (ROWS as f32 * CELL_SIZE) - text_dims.w / 2.0,
            std::f32::consts::PI * 1.5,
        );
        draw_outlined_text(
            canvas,
            right_x,
            BOARD_MARGIN + text_dims.w / 2.0,
            std::f32::consts::PI * 0.5,
        );

        Ok(())
    }
}

/// Implementation of the GGEZ EventHandler trait for GameState.
/// Handles game updates, drawing, and input events.
impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::WHITE);

        for row in 0..ROWS {
            for col in 0..COLS {
                self.draw_cell(&mut canvas, ctx, row, col)?;
            }
        }

        self.draw_turn_messages(&mut canvas, ctx)?;

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        if button == MouseButton::Left {
            let row = ((y - BOARD_MARGIN) / CELL_SIZE) as usize;
            let col = ((x - BOARD_MARGIN) / CELL_SIZE) as usize;
            if row < ROWS && col < COLS {
                self.place_tile(row, col);
            }
        }
        Ok(())
    }
}

/// Entry point of the game.
/// Sets up the game window and starts the game loop.
///
/// # Returns
/// * GameResult indicating success or failure
fn main() -> GameResult {
    let window_height = (ROWS as f32 * CELL_SIZE) + (2.0 * BOARD_MARGIN);

    let (ctx, event_loop) = ContextBuilder::new("color_war", "Author")
        .window_setup(ggez::conf::WindowSetup::default().title("Color War - Chain Reaction"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(
            (COLS as f32 * CELL_SIZE) + (2.0 * BOARD_MARGIN),
            window_height,
        ))
        .build()
        .expect("Failed to build ggez context");
    let state = GameState::new();
    event::run(ctx, event_loop, state)
}
