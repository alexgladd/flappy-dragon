use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

const ACCEL_GRAVITY: f32 = 9.8;
const VELOCITY_FLAP: f32 = -15.0;
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const PLAYER_OFFSET: i32 = 5;

struct Player {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            dx: 15.0,
            dy: 0.0,
        }
    }

    fn screen_y(&self) -> i32 {
        self.y.round() as i32
    }

    fn world_x(&self) -> i32 {
        self.x.round() as i32
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(PLAYER_OFFSET, self.screen_y(), YELLOW, BLACK, to_cp437('@'));
    }

    fn update(&mut self, delta_s: f32) {
        // gravity
        self.dy += ACCEL_GRAVITY * delta_s;

        self.y += self.dy * delta_s;
        self.x += self.dx * delta_s;

        if self.y < 0.0 {
            self.y = 0.0;
        }
    }

    fn flap(&mut self) {
        self.dy = VELOCITY_FLAP;
    }
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();

        Self {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20 - score),
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x + PLAYER_OFFSET;
        let half_size = self.size / 2;

        // draw the top half
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }

        // draw the bottom half
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
    }

    fn hit(&mut self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.world_x() == self.x;
        let player_above_gap = player.screen_y() < self.gap_y - half_size;
        let player_below_gap = player.screen_y() > self.gap_y + half_size;

        does_x_match && (player_above_gap || player_below_gap)
    }
}

struct State {
    player: Player,
    obstacle: Obstacle,
    mode: GameMode,
    score: i32,
    frame_time: f32,
}

impl State {
    fn new() -> Self {
        Self {
            player: Player::new(PLAYER_OFFSET as f32, SCREEN_HEIGHT as f32 / 2.0),
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            mode: GameMode::Menu,
            score: 0,
            frame_time: 0.0,
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);

        // self.frame_time += ctx.frame_time_ms;
        // if self.frame_time > FRAME_DURATION {
        //     self.frame_time = 0.0;
        //     self.player.update(ctx.frame_time_ms as f32 / 1000.0);
        // }

        let delta_s = ctx.frame_time_ms as f32 / 1000.0;
        self.player.update(delta_s);

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        self.player.render(ctx);
        self.obstacle.render(ctx, self.player.world_x());

        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, &format!("Score {}", self.score));

        if self.player.world_x() > self.obstacle.x + PLAYER_OFFSET {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.world_x() + SCREEN_WIDTH - PLAYER_OFFSET, self.score);
        }

        if self.player.screen_y() > SCREEN_HEIGHT || self.obstacle.hit(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(PLAYER_OFFSET as f32, SCREEN_HEIGHT as f32 / 2.0);
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        self.score = 0;
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "(P) Play game");
        ctx.print_centered(9, "(Q) Quit game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play again");
        ctx.print_centered(9, "(Q) Quit game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;
    
    main_loop(context, State::new())
}
