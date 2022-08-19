use crate::direction::Direction;
use crate::point::Point;

#[derive(Debug)]
pub struct Player {
    position: Point,
    direction: Option<Direction>,
}

impl Player {
    pub fn new(start: Point) -> Self {

        Self{ position: start, direction: None }
    }

    pub fn get_position(&self) -> Point {
        self.position.clone()
    }

    pub fn get_direction(&self) -> Option<Direction> {
        self.direction.clone()
    }

    pub fn move_player(&mut self) {
        self.position = self.position.transform(self.direction, 1);
    }

    pub fn set_direction(&mut self, direction: Option<Direction>) {
        self.direction = direction;
    }
}