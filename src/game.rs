use crate::player::Player;
use crate::point::Point;
use crate::direction::Direction;
use std::io::Stdout;
use std::time::{Duration, Instant};
use crossterm::{ExecutableCommand};
use crossterm::terminal::{Clear, ClearType, size, SetSize, enable_raw_mode, disable_raw_mode};
use crossterm::style::{SetForegroundColor, Print, ResetColor, Color};
use crossterm::cursor::{Show, MoveTo, Hide};
use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers, KeyEvent};
use crate::command::Command;
use rand::Rng;

const INTERVAL: Duration = Duration::from_millis(30);

#[derive(Debug)]
pub struct Game {
    stdout: Stdout,
    original_terminal_size: (u16, u16),
    width: u16,
    height: u16,
    rock: Vec<Point>,
    player: Player,
    rock_move_interval: u16,
    rock_gen_interval: u16,
    score: u32,
}

impl Game {
    pub fn new(stdout: Stdout, width: u16, height: u16, rock_move_interval: u16, rock_gen_interval: u16) -> Self {
        let original_terminal_size: (u16, u16) = size().unwrap();
        Self {
            stdout,
            original_terminal_size,
            width,
            height,
            rock: Vec::new(),
            player: Player::new(
                Point::new(width / 2, height - 1) // only position is required
            ),
            rock_move_interval,
            rock_gen_interval,
            score: 0,
        }
    }

    pub fn run(&mut self) {
        self.prepare_ui();
        self.render();

        let mut done = false;
        
        let mut rock_move_interval = self.rock_move_interval;
        let mut wait_for_rock_move = rock_move_interval;

        let mut rock_gen_interval = self.rock_gen_interval;
        let mut wait_for_rock_gen = rock_gen_interval;

        while !done {
            // let direction = self.player.get_direction();
            let now = Instant::now();
            wait_for_rock_gen -= 1;
            wait_for_rock_move -= 1;
            self.score += 1;

            while now.elapsed() < INTERVAL {
                if let Some(command) = self.get_command(INTERVAL - now.elapsed()) {
                    match command {
                        Command::Quit => {
                            done = true;
                            break;
                        }
                        Command::Go(towards) => {
                            self.player.set_direction(Some(towards));
                        }
                    }
                } else {
                    // self.player.set_direction(None);
                }
            }

            if self.has_collided_with_rock() {
                done = true;
            } else {
                let position = self.player.get_position();
                if !self.has_collided_with_wall() {
                    self.player.move_player();
                }
                
                if wait_for_rock_move <= 0 {
                    wait_for_rock_move = rock_move_interval;
                    self.move_rock();
                }

                if wait_for_rock_gen <= 0 {
                    wait_for_rock_gen = rock_gen_interval;
                    self.make_rock();
                }

                self.render();
            }
        }

        self.restore_ui();

        println!("Game Over! Your score is {}", self.score);
    }

    fn wait_for_key_event(&self, wait_for: Duration) -> Option<KeyEvent> {
        if poll(wait_for).ok()? {
            let event = read().ok()?;
            if let Event::Key(key_event) = event {
                return Some(key_event);
            }
        }

        None
    }

    fn get_command(&self, wait_for: Duration) -> Option<Command> {
        let key_event = self.wait_for_key_event(wait_for)?;

        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(Command::Quit),
            KeyCode::Char('c') | KeyCode::Char('C') =>
                if key_event.modifiers == KeyModifiers::CONTROL {
                    Some(Command::Quit)
                } else {
                    None
                }
            KeyCode::Right => Some(Command::Go(Direction::Right)),
            KeyCode::Left => Some(Command::Go(Direction::Left)),
            _ => None
        }
    }

    fn has_collided_with_rock(&self) -> bool {
        self.rock.contains(&self.player.get_position())
    }

    fn has_collided_with_wall(&self) -> bool {
        if let Some(direction) = self.player.get_direction() {
            let position = self.player.get_position();
            match direction {
                Direction::Right => position.x >= self.width-1,
                Direction::Left => position.x <= 0,
                _ => panic!("this situation cannot happen")
            }
        } else {
            true
        }
    }

    fn move_rock(&mut self) {
        let mut new_rock = Vec::new();

        for point in self.rock.iter() {
            let mut new_point = point.clone();
            new_point = new_point.transform(Some(Direction::Down), 1);
            if new_point.y < self.height {
                new_rock.push(new_point);
            }
        }

        self.rock = new_rock;
    }

    fn make_rock(&mut self) {
        loop {
            let random_x = rand::thread_rng().gen_range(0, self.width);
            let point = Point::new(random_x, 0);
            if !self.rock.contains(&point) {
                self.rock.push(point);
                break;
            }
        }
    }

    fn render(&mut self) {
        self.draw_borders();
        self.draw_background();
        self.draw_rock();
        self.draw_player();
        self.draw_score();
    }

    fn prepare_ui(&mut self) {
        enable_raw_mode().unwrap();
        self.stdout
            .execute(SetSize(self.width + 3, self.height + 3)).unwrap()
            .execute(Clear(ClearType::All)).unwrap()
            .execute(Hide).unwrap();
    }

    fn restore_ui(&mut self) {
        let (cols, rows) = self.original_terminal_size;
        self.stdout
            .execute(SetSize(cols, rows)).unwrap()
            .execute(Clear(ClearType::All)).unwrap()
            .execute(Show).unwrap()
            .execute(ResetColor).unwrap();
        disable_raw_mode().unwrap();
    }

    fn draw_score(&mut self) {
        self.stdout
                .execute(MoveTo(self.width / 2, self.height + 2)).unwrap()
                .execute(Print(self.score.to_string())).unwrap();
    }

    fn draw_player(&mut self) {
        let fg = SetForegroundColor(match (self.score / 10) % 3 { // 후에 점수에 따라 바뀌도록 수정 해야함.
            0 => Color::Green,
            1 => Color::Cyan,
            _ => Color::Yellow
        });
        self.stdout.execute(fg).unwrap();

        let body = self.player.get_position();
        let symbol = "+";

        self.stdout
                .execute(MoveTo(body.x + 1, body.y + 1)).unwrap()
                .execute(Print(symbol)).unwrap();
    }

    fn draw_rock(&mut self) {
        self.stdout.execute(SetForegroundColor(Color::White)).unwrap();

        for rock in self.rock.iter() {
            self.stdout
                .execute(MoveTo(rock.x + 1, rock.y + 1)).unwrap()
                .execute(Print("•")).unwrap();
        }
    }

    fn draw_background(&mut self) {
        self.stdout.execute(ResetColor).unwrap();

        for y in 1..self.height + 1 {
            for x in 1..self.width + 1 {
                self.stdout
                    .execute(MoveTo(x, y)).unwrap()
                    .execute(Print(" ")).unwrap();
            }
        }
    }

    fn draw_borders(&mut self) {
        self.stdout.execute(SetForegroundColor(Color::DarkGrey)).unwrap();

        for y in 0..self.height + 2 {
            self.stdout
                .execute(MoveTo(0, y)).unwrap()
                .execute(Print("#")).unwrap()
                .execute(MoveTo(self.width + 1, y)).unwrap()
                .execute(Print("#")).unwrap();
        }

        for x in 0..self.width + 2 {
            self.stdout
                .execute(MoveTo(x, 0)).unwrap()
                .execute(Print("#")).unwrap()
                .execute(MoveTo(x, self.height + 1)).unwrap()
                .execute(Print("#")).unwrap();
        }

        self.stdout
            .execute(MoveTo(0, 0)).unwrap()
            .execute(Print("#")).unwrap()
            .execute(MoveTo(self.width + 1, self.height + 1)).unwrap()
            .execute(Print("#")).unwrap()
            .execute(MoveTo(self.width + 1, 0)).unwrap()
            .execute(Print("#")).unwrap()
            .execute(MoveTo(0, self.height + 1)).unwrap()
            .execute(Print("#")).unwrap();
    }
}