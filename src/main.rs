use chess::game_turn;

use ggez::{conf, event, graphics, Context, ContextBuilder, GameError, GameResult, input::mouse};
use std::{collections::HashMap, env, path, fmt::{self}};

/// A chess board is 8x8 tiles.
const GRID_SIZE: u8 = 8;
/// Suitable size of each tile.
const GRID_CELL_SIZE: (u16, u16) = (90, 90);

/// Size of the application window.
const SCREEN_SIZE: (f32, f32) = (
    ((6.0 + GRID_SIZE as f32) * GRID_CELL_SIZE.0 as f32), // window width
    GRID_SIZE as f32 * GRID_CELL_SIZE.1 as f32, // window height
);

// GUI Color representations
const WHITE: graphics::Color =
    graphics::Color::new(250.0 / 255.0, 240.0 / 255.0, 222.0 / 255.0, 1.0);
const BLACK: graphics::Color =
    graphics::Color::new(236.0 / 255.0, 95.0 / 255.0, 153.0 / 255.0, 1.0);

// GUI logic and event implementation structure.


fn get_algebraic_notation(x_pos: i32, y_pos: i32) -> String {
    let col_names = "abcdefgh".to_string();
    let col_name = col_names.chars().nth(x_pos as usize).expect("Blimey! Unable to find this col!");
    format!("{}{}", col_name, 8 - y_pos)
}

struct Game {
    fen: String,
    turn: char,
    board: Vec<Vec<char>>,
}
impl Game {
    
    fn parse_fen(fen: String) -> Game {
        let fen_vec = fen.split(' ').collect::<Vec<&str>>();
        //Split the FEN into its constituent parts

        let board_state_vec = fen_vec[0].split('/').collect::<Vec<&str>>();
        let mut row = vec![];
        let mut board_state = vec![];
        for single_row in board_state_vec {
            for character in single_row.chars() {
                //assuming valid FEN (only characters and numbers)
                const RADIX: u32 = 10;
                if character.is_numeric() {
                    for _i in 0..character
                        .to_digit(RADIX)
                        .expect("Could not convert char to int")
                    {
                        row.push('*');
                    }
                } else {
                    row.push(character);
                }
            }
            board_state.push(row.clone());
            row.retain(|_x| false); // empty the vector
        }
        Game {
            fen: fen.clone(),
            turn: fen_vec[1].chars().collect::<Vec<char>>()[0],
            board: board_state
        }
    }

    fn update_fen(&mut self, fen: String) {
        let fen_vec = fen.split(' ').collect::<Vec<&str>>();
        //Split the FEN into its constituent parts

        let board_state_vec = fen_vec[0].split('/').collect::<Vec<&str>>();
        let mut row = vec![];
        let mut board_state = vec![];
        for single_row in board_state_vec {
            for character in single_row.chars() {
                //assuming valid FEN (only characters and numbers)
                const RADIX: u32 = 10;
                if character.is_numeric() {
                    for _i in 0..character
                        .to_digit(RADIX)
                        .expect("Could not convert char to int")
                    {
                        row.push('*');
                    }
                } else {
                    row.push(character);
                }
            }
            board_state.push(row.clone());
            row.retain(|_x| false); // empty the vector
        }
        self.fen = fen.clone();
        self.turn = fen_vec[1].chars().collect::<Vec<char>>()[0];
        self.board = board_state;
    }
    fn new() -> Game {
        Self::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())
    }    
}
impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut board_state_string = String::new();
        for i in &self.board {
            board_state_string = format!("{}\n{:?}", board_state_string, i);
        }
        write!(
            f,
            "Current FEN: {}\nCurrent turn: {}\nCurrent board state: {}",
            self.fen, self.turn, board_state_string,
        )
    }
}
struct AppState {
    sprites: HashMap<char, graphics::Image>, // For easy access to the apropriate PNGs
    game: Game,
    piece_picked_up: Vec<i32>,
    debug: bool,
}

impl AppState {
    // Initialise new application, i.e. initialise new game and load resources.
    fn new(ctx: &mut Context) -> GameResult<AppState> {

        let state = AppState {
            sprites: AppState::load_sprites(ctx),
            game: Game::new(),
            piece_picked_up: vec![],
            debug: false // change this if debug information is needed in GUI
        };

        Ok(state)
    }
    #[rustfmt::skip] // Skips formatting on this function (not recommended)
                     /// Loads chess piese images into hashmap, for ease of use.
    fn load_sprites(ctx: &mut Context) -> HashMap<char, graphics::Image> {

        [
            (('k'), "/black_king.png".to_string()),
            (('q'), "/black_queen.png".to_string()),
            (('r'), "/black_rook.png".to_string()),
            (('p'), "/black_pawn.png".to_string()),
            (('b'), "/black_bishop.png".to_string()),
            (('n'), "/black_knight.png".to_string()),
            (('K'), "/white_king.png".to_string()),
            (('Q'), "/white_queen.png".to_string()),
            (('R'), "/white_rook.png".to_string()),
            (('P'), "/white_pawn.png".to_string()),
            (('B'), "/white_bishop.png".to_string()),
            (('N'), "/white_knight.png".to_string())
        ]
            .iter()
            .map(|(piece, path)| {
                (*piece, graphics::Image::new(ctx, path).unwrap())
            })
            .collect::<HashMap<char, graphics::Image>>()
    }
}

// This is where we implement the functions that ggez requires to function
impl event::EventHandler<GameError> for AppState {
    /// For updating game logic, which front-end doesn't handle.
    /// It won't be necessary to touch this unless you are implementing something that's not triggered by the user, like a clock
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    /// Draw interface, i.e. draw game board
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mouse_position = mouse::position(ctx);
        // clear interface with gray background colour
        graphics::clear(ctx, [0.5, 0.5, 0.5, 1.0].into());

        // create text representation
        let debug_text = graphics::Text::new(
            graphics::TextFragment::from(format!("Debug information:\n{:?}", self.game))
                .scale(graphics::PxScale { x: 15.0, y: 15.0 }),
        );



        // get size of text
        let debug_text_dimensions = debug_text.dimensions(ctx);
        let debug_text_position = [(SCREEN_SIZE.0 - debug_text_dimensions.w as f32), (SCREEN_SIZE.1 - debug_text_dimensions.h as f32)];
        // create background rectangle with OFF BLACK coulouring
        let debug_background_box = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(
                debug_text_position[0],
                debug_text_position[1],
                debug_text_dimensions.w as f32 + 16.0,
                debug_text_dimensions.h as f32,
            ),
            [33.0/255.0, 33.0/255.0, 33.0/255.0, 1.0].into(),
        )?;
        if self.debug {
            // draw background
            graphics::draw(ctx, &debug_background_box, graphics::DrawParam::default())
            .expect("Failed to draw background.");
        }
        let restart_text = graphics::Text::new(
            graphics::TextFragment::from("[RESTART]")
                    .scale(graphics::PxScale{x: 30.0, y: 30.0}),
        );
        let restart_text_dimensions = restart_text.dimensions(ctx);
        let restart_text_position = [(GRID_CELL_SIZE.0 as f32 * 11.0) - (restart_text_dimensions.w / 2.0), (GRID_CELL_SIZE.1 as f32 * 2.5) - (restart_text_dimensions.h / 2.0)];

        // create Restart button
        let mut color = [33.0/255.0, 33.0/255.0, 33.0/255.0, 1.0];
        if mouse_position.x >= GRID_CELL_SIZE.0 as f32 * 10.0 && mouse_position.x <= GRID_CELL_SIZE.0 as f32 * 12.0 &&
            mouse_position.y >= GRID_CELL_SIZE.1 as f32 * 2.0 && mouse_position.y <= GRID_CELL_SIZE.1 as f32 * 3.0 {
                color = [153.0/255.0, 153.0/255.0, 153.0/255.0, 1.0]
            }

        let restart_button = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(
                GRID_CELL_SIZE.0 as f32 * 10.0,
                GRID_CELL_SIZE.1 as f32 * 2.0,
                (GRID_CELL_SIZE.0 * 2).into(),
                GRID_CELL_SIZE.1.into(),
            ),
            color.into(),
        )?;

        graphics::draw(ctx, &restart_button, graphics::DrawParam::default()).expect("Failed to draw restart button background.");

        // draw [RESTART] text

        const F7: f32 = 0.96862745;
        graphics::draw(
            ctx,
            
            &restart_text,
            graphics::DrawParam::default()
                .color([F7, F7, F7, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: restart_text_position[0],
                    y: restart_text_position[1],                    
                }),
        )
        .expect("Failed to draw restart text.");

        // Draw turn indicator text 
        let mut turn_indicator = String::new();
        match self.game.turn {
            'w' => turn_indicator = "White".to_string(),
            'b' => turn_indicator = "Black".to_string(),
            _ => panic!("Oh my goodness! This color does not exist!")
        }
        let turn_indicator_text = graphics::Text::new(
            graphics::TextFragment::from(format!(
                "It is {}'s turn.",
                turn_indicator,
            )).scale(graphics::PxScale {x: 30.0, y: 30.0})
        );
        // let turn_indicator_text_dimensions = turn_indicator_text.dimensions(ctx);
        let turn_indicator_text_position = [(GRID_CELL_SIZE.0 as f32 * 9.5), (GRID_CELL_SIZE.1 as f32 * 3.5)];
                graphics::draw(
            ctx,
            
            &turn_indicator_text,
            graphics::DrawParam::default()
                .color([F7, F7, F7, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: turn_indicator_text_position[0],
                    y: turn_indicator_text_position[1],                    
                }),
        )
        .expect("Failed to draw restart text.");


        // draw grid
        for row in 0..8 {
            for col in 0..8 {
                // draw tile
                let rectangle = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    graphics::Rect::new_i32(
                        col * GRID_CELL_SIZE.0 as i32,
                        row * GRID_CELL_SIZE.1 as i32,
                        GRID_CELL_SIZE.0 as i32,
                        GRID_CELL_SIZE.1 as i32,
                    ),
                    match col % 2 {
                        0 => {
                            if row % 2 == 0 {
                                WHITE
                            } else {
                                BLACK
                            }
                        }
                        _ => {
                            if row % 2 == 0 {
                                BLACK
                            } else {
                                WHITE
                            }
                        }
                    },
                )
                .expect("Failed to create tile.");
                graphics::draw(ctx, &rectangle, graphics::DrawParam::default())
                    .expect("Failed to draw tiles.");

                
            }
        }
        
        // draw pieces
        for row in 0..8 {
            for col in 0..8 {
                if self.game.board[row as usize][col as usize] != '*' {
                    let mut x_pos = col as f32 * GRID_CELL_SIZE.0 as f32;
                    let mut y_pos = row as f32 * GRID_CELL_SIZE.1 as f32;
                    if !self.piece_picked_up.is_empty() {
                        if col != self.piece_picked_up[0] || row != self.piece_picked_up[1] {
                            graphics::draw(
                                ctx,
                                self.sprites.get(&self.game.board[row as usize][col as usize]).unwrap(),
                                graphics::DrawParam::default()
                                    .scale([2.0, 2.0]) // Tile size is 90 pixels, while image sizes are 45 pixels.
                                    .dest([
                                        x_pos,
                                        y_pos,
                                    ]),
                            ).expect("Failed to draw piece.");
                        }
                    } else {
                        graphics::draw(
                            ctx,
                            self.sprites.get(&self.game.board[row as usize][col as usize]).unwrap(),
                            graphics::DrawParam::default()
                                .scale([2.0, 2.0]) // Tile size is 90 pixels, while image sizes are 45 pixels.
                                .dest([
                                    x_pos,
                                    y_pos,
                                ]),
                        )
                        .expect("Failed to draw piece.");
                    }
                }
            }
        }
        // draw picked up piece last
        if !self.piece_picked_up.is_empty() {
            for row in 0..8 {
                for col in 0..8 {
                    if col == self.piece_picked_up[0] && row == self.piece_picked_up[1] {
                        // let mouse_position = mouse::position(ctx);
                        let x_pos = mouse_position.x - 20.0;
                        let y_pos = mouse_position.y - 20.0;
                        graphics::draw(
                            ctx,
                            self.sprites.get(&self.game.board[row as usize][col as usize]).unwrap(),
                            graphics::DrawParam::default()
                                .scale([2.0, 2.0]) // Tile size is 90 pixels, while image sizes are 45 pixels.
                                .dest([
                                    x_pos,
                                    y_pos,
                                ]),
                        ).expect("Failed to draw picked up piece.");
                    }
                }
            }
        }
        
        // Draw Restart button text
        
        
        // draw debug text with off white colouring and center position
        if self.debug {
            const F7: f32 = 0.96862745;
            graphics::draw(
                ctx,
                
                &debug_text,
                graphics::DrawParam::default()
                    .color([F7, F7, F7, 1.0].into())
                    .dest(ggez::mint::Point2 {
                        x: debug_text_position[0],
                        y: debug_text_position[1],
                    }),
            )
            .expect("Failed to draw text.");
        }







        // render updated graphics
        graphics::present(ctx).expect("Failed to update graphics.");
        Ok(())
    }

    /// Update game on mouse click
    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) {
        let pos = mouse::position(ctx);
        let board_pos_x = (pos.x / GRID_CELL_SIZE.0 as f32).floor() as i32;
        let board_pos_y = (pos.y / GRID_CELL_SIZE.0 as f32).floor() as i32;
        if button == event::MouseButton::Left {
            
            if board_pos_x <= 7 { // this means that the click was within board boundaries
                println!("x coordinate: {}, y coordinate: {}, algebraic notation: {}", board_pos_x, board_pos_y, get_algebraic_notation(board_pos_x, board_pos_y));
                if self.piece_picked_up.is_empty() { // This means a piece hasnt been picked up
                    self.piece_picked_up = vec![board_pos_x, board_pos_y]; // set piece picked up flag
                } else { // only run if a piece has been picked up
                    let algebraic_coordinate_source = get_algebraic_notation(self.piece_picked_up[0], self.piece_picked_up[1]);
                    let algebraic_coordinate_target  = get_algebraic_notation(board_pos_x, board_pos_y);
                    let action = format!("{} {}", algebraic_coordinate_source, algebraic_coordinate_target);
                    println!("Action taken: {}", action);
                    self.game.update_fen(game_turn(self.game.fen.clone(), action));
                    self.piece_picked_up.retain(|_| false); //empty the vector
                }
            } else {
                if (board_pos_x == 10 || board_pos_x == 11) && board_pos_y == 2 {
                    self.game = Game::new();
                }
            }
        }
    }
}

pub fn main() -> GameResult {
    let resource_dir = path::PathBuf::from("./resources");

    let context_builder = ContextBuilder::new("schack", "viola")
        .add_resource_path(resource_dir) // Import image files to GGEZ
        .window_setup(
            conf::WindowSetup::default()
                .title("BestDamnSchackMonitor (BDSM)") // Set window title
                .icon("/datasektionen.png"), // Set application icon
        )
        .window_mode(
            conf::WindowMode::default()
                .dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1) // Set window dimensions
                .resizable(false), // Fixate window size
        );
    let (mut contex, mut event_loop) = context_builder.build().expect("Failed to build context.");

    let state = AppState::new(&mut contex).expect("Failed to create state.");
    event::run(contex, event_loop, state) // Run window event loop
    }