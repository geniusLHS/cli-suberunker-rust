use crate::direction::Direction;

pub enum Command {
    Quit,
    Go(Direction),
}