use rand::prelude::*;
use rusty_engine::prelude::*;

const GRAVITY: f32 = 0.3;
const JUMP_VELOCITY: f32 = -5.0;
const BIRD_HEIGHT: f32 = 12.0;
const PIPE_WIDTH: f32 = 52.0;
const PIPE_GAP: f32 = 125.0;
const PIPE_DIST: f32 = 200.0;
const BASE_HEIGHT: f32 = 112.0;
const PIPE_HEIGHT: f32 = 320.0;
const PIPE_SPEED: f32 = 2.0;
const VIEWPORT_HEIGHT: f32 = 400.0;

struct GameState {
    bird_velocity: f32,
    lost: bool,
    pipes: Vec<PipePair>,
    score: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            bird_velocity: 0.0,
            lost: false,
            pipes: Vec::new(),
            score: 0,
        }
    }
}

struct PipePair {
    pipes: Vec<Pipe>,
}

struct Pipe {
    x: f32,
    y: f32,
    scored: bool,
}

impl Pipe {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            scored: false,
        }
    }
}

fn main() {
    let mut game_state = GameState::default();
    let mut game = Game::new();

    game.window_settings(WindowDescriptor {
        width: 288.0,
        height: 512.0,
        title: ("Flappy Bird".to_string()),
        ..Default::default()
    });

    let background = game.add_sprite("background", "sprite/flappy/background.png");
    background.layer = 0.0;
    background.collision = false;

    let base = game.add_sprite("base", "sprite/flappy/base.png");
    base.layer = 11.0;
    base.translation.y = -256.0 + BASE_HEIGHT / 2.0;
    base.collision = false;

    init_sprites(&mut game, &mut game_state);

    game.add_logic(game_logic);
    game.run(game_state);
}

fn game_logic(engine: &mut Engine, game_state: &mut GameState) {
    // If the game is lost and 'R' is pressed, reset the game state
    if game_state.lost && (engine.keyboard_state.pressed(KeyCode::R)) {
        cleanup_sprites(engine, game_state);
        *game_state = GameState::default();
        init_sprites(engine, game_state);

        return;
    }

    // game logic goes here
    if game_state.lost {
        return;
    }

    // Update bird's velocity
    game_state.bird_velocity += GRAVITY;

    // Update bird's position
    let bird = engine.sprites.get_mut("bird").unwrap();
    bird.translation.y -= game_state.bird_velocity;
    if bird.translation.y < -256.0 + BIRD_HEIGHT + BASE_HEIGHT
        || bird.translation.y > 256.0 - BIRD_HEIGHT
    {
        game_state.lost = true;
        engine.audio_manager.play_sfx(SfxPreset::Jingle3, 0.5);
    }

    // Handle space bar for jumping
    if (engine.keyboard_state.pressed(KeyCode::Space)
        || engine.keyboard_state.pressed(KeyCode::W)
        || engine.keyboard_state.pressed(KeyCode::Up))
        && game_state.bird_velocity > 0.0
    {
        game_state.bird_velocity = JUMP_VELOCITY;
    }

    // Check for clearing a pipe
    let score_text = engine.texts.get_mut("score").unwrap();
    let mut should_increment_score = false;

    let mut new_pipes = Vec::new();
    let mut sprite_updates = Vec::new();

    for (i, pipe_pair) in game_state.pipes.iter_mut().enumerate() {
        for pipe in &mut pipe_pair.pipes {
            pipe.x -= PIPE_SPEED;
            // pipe.scored = false; // Reset the scored flag for each pipe

            // Check if the bird has passed the current pipe and it hasn't been scored yet
            if bird.translation.x > pipe.x + PIPE_WIDTH && !pipe.scored {
                pipe.scored = true;
                should_increment_score = true;
            }

            // Store the updates needed to be done on the sprites
            let sprite_name = if pipe.y < 0.0 {
                format!("top_pipe{}", i)
            } else {
                format!("bot_pipe{}", i)
            };
            sprite_updates.push((sprite_name, pipe.x));
        }

        if pipe_pair.pipes[0].x < -PIPE_WIDTH - PIPE_DIST {
            let new_offset = 288.0 + 10.0;
            new_pipes.push((i, get_random_pipe(new_offset)));
        }
    }

    // Update the sprites' translation
    for (sprite_name, x) in sprite_updates {
        if let Some(sprite) = engine.sprites.get_mut(&sprite_name) {
            sprite.translation.x = x;
        }
    }

    // Replace the pipes
    for (i, new_pipe) in new_pipes {
        game_state.pipes[i] = new_pipe;
    }

    // Increment the score if the bird has passed a pipe
    if should_increment_score {
        game_state.score += 1;
        engine.audio_manager.play_sfx(SfxPreset::Confirmation1, 0.5);
        score_text.value = format!("Score: {}", game_state.score);
        score_text.font_size = 30.0;
        score_text.translation.y = 216.0;
    }

    // Deal with collisions
    for event in engine.collision_events.drain(..) {
        if !event.pair.either_contains("bird") {
            continue;
        }
        if !game_state.lost {
            game_state.lost = true;
            engine.audio_manager.play_sfx(SfxPreset::Jingle3, 0.5);
        }
    }

    // Handle losing
    if game_state.lost {
        let game_over = engine.add_text("game_over", "Game Over");
        game_over.font_size = 60.0;
        game_over.translation.y = 20.0;
        let score_text = engine.add_text("score", &format!("Score: {}", game_state.score));
        score_text.font_size = 30.0;
        score_text.translation.y = -20.0;
        engine.audio_manager.stop_music();
        // engine.collision_events.clear();
    }
}

fn get_random_pipe(offset: f32) -> PipePair {
    let gap_y =
        thread_rng().gen_range(0.0..(VIEWPORT_HEIGHT * 0.6 - PIPE_GAP)) + VIEWPORT_HEIGHT * 0.2;
    PipePair {
        pipes: vec![
            Pipe::new(288.0 + 10.0 + offset, gap_y + PIPE_HEIGHT - 150.0),
            Pipe::new(288.0 + 10.0 + offset, gap_y - PIPE_GAP - 150.0),
        ],
    }
}

fn cleanup_sprites(engine: &mut Engine, game_state: &mut GameState) {
    // Remove existing sprites
    engine.sprites.remove("bird");
    for i in 0..game_state.pipes.len() {
        engine.sprites.remove(&format!("top_pipe{}", i));
        engine.sprites.remove(&format!("bot_pipe{}", i));
    }

    // Remove game_over text if it exists
    engine.texts.remove("game_over");
    engine.texts.remove("score");

    // Clear the pipes vector in the game_state
    game_state.pipes.clear();
}

fn init_sprites(engine: &mut Engine, game_state: &mut GameState) {
    // Add bird back in its initial position
    let bird = engine.add_sprite("bird", "sprite/flappy/yellowbird_default.png");
    bird.translation.y = BASE_HEIGHT / 2.0;
    bird.layer = 10.0;
    bird.collision = true;
    bird.layer = 5.0;

    for i in 0..4 {
        let pipe_pair = get_random_pipe(i as f32 * PIPE_DIST);
        let top_pipe = engine.add_sprite(format!("top_pipe{}", i), "sprite/flappy/top_pipe.png");
        top_pipe.translation.x = pipe_pair.pipes[0].x;
        top_pipe.translation.y = pipe_pair.pipes[0].y;
        top_pipe.layer = 10.0;
        top_pipe.collision = true;
        let bot_pipe = engine.add_sprite(format!("bot_pipe{}", i), "sprite/flappy/bot_pipe.png");
        bot_pipe.translation.x = pipe_pair.pipes[1].x;
        bot_pipe.translation.y = pipe_pair.pipes[1].y;
        bot_pipe.layer = 10.0;
        bot_pipe.collision = true;

        game_state.pipes.push(pipe_pair);
    }

    // Start background music
    engine
        .audio_manager
        .play_music(MusicPreset::WhimsicalPopsicle, 0.2);

    // Score
    let score_text = engine.add_text("score", "Score: 0");
    score_text.font_size = 30.0;
    score_text.translation.y = 216.0;
}
