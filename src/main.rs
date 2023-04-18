use macroquad::prelude::*;

// width and height respectively
const PLAYER_SIZE: Vec2 = Vec2::from_array([150f32, 40f32]);
const BLOCK_SIZE: Vec2 = Vec2::from_array([100f32, 40f32]);
const BALL_SIZE: f32 = 50f32;

const PLAYER_SPEED: f32 = 700f32;
const BALL_SPEED: f32 = 450f32;

enum GameState {
    Menu,
    Game,
    Win,
    Dead,
}

struct Player {
    rect: Rect,
}

struct Block {
    rect: Rect,
    lives: i32,
    block_type: BlockType,
}

#[derive(PartialEq)]
enum BlockType {
    Regular,
    SpawnBall,
}

struct Ball {
    rect: Rect,
    vel: Vec2,
}

impl Ball {
    pub fn new(pos: Vec2) -> Self {
        Self {
            rect: Rect::new(pos.x, pos.y, BALL_SIZE, BALL_SIZE),
            vel: vec2(
                rand::gen_range(-1.5f32, 1.5f32),
                rand::gen_range(-1.5f32, -1f32),
            ),
        }
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, GREEN)
    }

    pub fn update(&mut self, dt: f32) {
        self.rect.x += self.vel.x * dt * BALL_SPEED;
        self.rect.y += self.vel.y * dt * BALL_SPEED;

        // Horizontal velocity resolve
        if self.rect.x < 0f32 || self.rect.x > screen_width() - self.rect.w {
            self.vel.x *= -1f32;
        }

        // Vertical velocity resolve
        if self.rect.y <= 0f32 {
            self.vel.y *= -1f32;
        }

        self.draw();
    }
}

impl Block {
    pub fn new(pos: Vec2, block_type: BlockType, lives: i32) -> Self {
        Self {
            rect: Rect::new(pos.x, pos.y, BLOCK_SIZE.x, BLOCK_SIZE.y),
            lives,
            block_type,
        }
    }
    pub fn draw(&self) {
        let color = match self.block_type {
            BlockType::Regular => match self.lives {
                2 => GOLD,
                _ => RED,
            },
            BlockType::SpawnBall => PURPLE,
        };
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, color);
    }
    pub fn update(&self) {
        self.draw();
    }
}

impl Player {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                screen_width() * 0.5 - PLAYER_SIZE.x * 0.5,
                screen_height() - 100.0,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y,
            ),
        }
    }

    pub fn update(&mut self, dt: f32) {
        let x_move = match (
            is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
            is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
        ) {
            (true, false) => -1f32,
            (false, true) => 1f32,
            _ => 0f32,
        };
        self.rect.x += x_move * dt * PLAYER_SPEED;

        // Clamp to screen screen widths
        if self.rect.x + self.rect.w > screen_width() {
            self.rect.x = screen_width() - self.rect.w;
        } else if self.rect.x < 0f32 {
            self.rect.x = 0f32;
        }

        self.draw();
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, BLUE);
    }
}

fn init_blocks(blocks: &mut Vec<Block>) {
    let (blocks_wide, blocks_high) = (8, 8);

    let padding = 15f32;
    let total_block_size = BLOCK_SIZE + vec2(padding, padding);

    let board_start_pos = vec2(
        (screen_width() - (blocks_wide as f32 * total_block_size.x)) * 0.5f32,
        50f32,
    );

    for i in 0..blocks_wide * blocks_high {
        let block_x = (i % blocks_wide) as f32 * total_block_size.x;
        let block_y = (i / blocks_wide) as f32 * total_block_size.y;
        blocks.push(Block::new(
            board_start_pos + vec2(block_x, block_y),
            BlockType::Regular,
            rand::gen_range(1, 3),
        ));
    }

    // now we can allocated randomly our spawnableBall blocks
    for _ in 0..8 {
        let rand_index = rand::gen_range(0, blocks.len() - 1);
        blocks[rand_index].block_type = BlockType::SpawnBall;
        blocks[rand_index].lives = 1;
    }
}

fn resolve_collision(a: &mut Rect, vel: &mut Vec2, b: &Rect) -> bool {
    if let Some(intersection_rect) = a.intersect(*b) {
        let a_center = a.center();
        let b_center = b.center();
        let a_to_b = b_center - a_center;

        let signum = a_to_b.signum();

        // this checks if it hits the top of the block or the sides
        match intersection_rect.w > intersection_rect.h {
            true => {
                // bounce on y
                a.y -= signum.y * intersection_rect.h;
                vel.y *= -1f32;
            }
            false => {
                // bounce on x
                a.x -= signum.x * intersection_rect.w;
                vel.x *= -1f32;
            }
        }
        return true;
    }
    false
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Breakout".to_owned(),
        // fullscreen: true,
        window_height: 800,
        window_width: 1000,
        ..Default::default()
    }
}

fn spawn_ball(balls: &mut Vec<Ball>, position: Vec2) {
    balls.push(Ball::new(position));
}

#[macroquad::main(window_conf)]
async fn main() {
    let font = load_ttf_font("res/Poppins-SemiBold.ttf").await.unwrap();

    let mut game_state = GameState::Menu;

    let mut score = 0;
    let mut player_lives = 3;

    let mut player = Player::new();
    let mut blocks: Vec<Block> = Vec::new();
    let mut balls: Vec<Ball> = Vec::new();

    loop {
        clear_background(WHITE);
        player.update(get_frame_time());

        let mut to_spawn: Vec<Vec2> = vec![];
        for block in blocks.iter_mut() {
            // now resolve collision with all balls
            for ball in balls.iter_mut() {
                if resolve_collision(&mut ball.rect, &mut ball.vel, &block.rect) {
                    block.lives -= 1;
                    if block.lives <= 0 {
                        score += 1;
                        if block.block_type == BlockType::SpawnBall {
                            let block_position = vec2(
                                block.rect.x + block.rect.w * 0.5,
                                block.rect.y + block.rect.h * 0.5,
                            );
                            to_spawn.push(block_position);
                        }
                    }
                }
            }
            block.update();
        }
        for position_to_spawn_ball in to_spawn.into_iter() {
            spawn_ball(&mut balls, position_to_spawn_ball);
        }

        let prev_num_balls = balls.len();
        balls.retain(|ball| ball.rect.y < screen_height());
        let post_num_balls = balls.len();

        if post_num_balls < prev_num_balls && post_num_balls == 0 {
            player_lives -= 1;
        }

        blocks.retain(|block| block.lives > 0);

        for ball in balls.iter_mut() {
            resolve_collision(&mut ball.rect, &mut ball.vel, &player.rect);
            ball.update(get_frame_time());
        }

        let font_size = 30u16;

        match game_state {
            GameState::Menu => {
                let welcome_text_dimensions =
                    measure_text("Press space to play!", Some(font), font_size, 1.0);
                player_lives = 3;
                score = 0;

                draw_text_ex(
                    "Press space to play!",
                    screen_width() * 0.5f32 - welcome_text_dimensions.width * 0.5,
                    screen_height() * 0.5f32 - welcome_text_dimensions.height * 0.5,
                    TextParams {
                        font,
                        font_size,
                        color: BLACK,
                        ..Default::default()
                    },
                );

                if is_key_pressed(KeyCode::Space) {
                    init_blocks(&mut blocks);
                    game_state = GameState::Game;
                    let player_position =
                        vec2(player.rect.x + player.rect.w * 0.5, player.rect.y - 40.0);
                    spawn_ball(&mut balls, player_position);
                }
            }

            GameState::Game => {
                let score_text_dimensions =
                    measure_text(&format!("Score: {}", score), Some(font), font_size, 1.0);

                draw_text_ex(
                    &format!("  Lives: {player_lives}"),
                    0.0,
                    40.0,
                    TextParams {
                        font,
                        font_size,
                        color: BLACK,
                        ..Default::default()
                    },
                );

                draw_text_ex(
                    &format!("Score: {}", score),
                    screen_width() * 0.5f32 - score_text_dimensions.width * 0.5,
                    40.0,
                    TextParams {
                        font,
                        font_size,
                        color: BLACK,
                        ..Default::default()
                    },
                );

                if blocks.len() == 0 {
                    game_state = GameState::Win;
                }

                if player_lives <= 0 {
                    game_state = GameState::Dead;
                }

                if balls.len() <= 0 {
                    if is_key_pressed(KeyCode::Space) {
                        let player_position =
                            vec2(player.rect.x + player.rect.w * 0.5, player.rect.y - 40.0);
                        spawn_ball(&mut balls, player_position);
                    }
                }
            }
            GameState::Dead => {
                let death_text_dimensions = measure_text(
                    &format!("You died :( but you got a score of {}!", score),
                    Some(font),
                    font_size,
                    1.0,
                );
                let restart_text_dimensions =
                    measure_text("Press space to return to menu!", Some(font), font_size, 1.0);

                draw_text_ex(
                    &format!("You died :( but you got a score of {}!", score),
                    screen_width() * 0.5f32 - death_text_dimensions.width * 0.5,
                    screen_height() * 0.5f32 - death_text_dimensions.height * 0.5,
                    TextParams {
                        font,
                        font_size,
                        color: BLACK,
                        ..Default::default()
                    },
                );

                draw_text_ex(
                    "Press space to return to menu!",
                    screen_width() * 0.5f32 - restart_text_dimensions.width * 0.5,
                    screen_height() * 0.8f32 - restart_text_dimensions.height * 0.5,
                    TextParams {
                        font,
                        font_size,
                        color: BLACK,
                        ..Default::default()
                    },
                );

                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Menu;
                }
            }
            GameState::Win => {
                let death_text_dimensions = measure_text(
                    &format!("You died :( but you got a score of {}!", score),
                    Some(font),
                    font_size,
                    1.0,
                );
                let restart_text_dimensions =
                    measure_text("Press space to return to menu!", Some(font), font_size, 1.0);

                draw_text_ex(
                    &format!("You win! :) You got a score of {}!", score),
                    screen_width() * 0.5f32 - death_text_dimensions.width * 0.5,
                    screen_height() * 0.5f32 - death_text_dimensions.height * 0.5,
                    TextParams {
                        font,
                        font_size,
                        color: BLACK,
                        ..Default::default()
                    },
                );

                draw_text_ex(
                    "Press space to return to menu!",
                    screen_width() * 0.5f32 - restart_text_dimensions.width * 0.5,
                    screen_height() * 0.8f32 - restart_text_dimensions.height * 0.5,
                    TextParams {
                        font,
                        font_size,
                        color: BLACK,
                        ..Default::default()
                    },
                );

                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Menu;
                }
            }
        }

        next_frame().await;
    }
}
