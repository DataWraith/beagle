use std::fmt::{Display, Formatter, Result};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
    Stay,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Direction::North => write!(f, "{}", "North"),
            Direction::East => write!(f, "{}", "East"),
            Direction::South => write!(f, "{}", "South"),
            Direction::West => write!(f, "{}", "West"),
            Direction::Stay => write!(f, "{}", "Stay"),
        }
    }
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::Stay
    }
}

impl From<u8> for Direction {
    fn from(x: u8) -> Direction {
        match x {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            3 => Direction::West,
            4 => Direction::Stay,
            _ => unreachable!(),
        }
    }
}

impl From<&'static str> for Direction {
    fn from(s: &'static str) -> Direction {
        match s {
            "North" => Direction::North,
            "East" => Direction::East,
            "South" => Direction::South,
            "West" => Direction::West,
            "Stay" => Direction::Stay,
            _ => unreachable!(),
        }
    }
}

impl From<String> for Direction {
    fn from(s: String) -> Direction {
        match s.as_ref() {
            "North" => Direction::North,
            "East" => Direction::East,
            "South" => Direction::South,
            "West" => Direction::West,
            "Stay" => Direction::Stay,
            _ => unreachable!(),
        }
    }
}

impl Into<&'static str> for Direction {
    fn into(self) -> &'static str {
        match self {
            Direction::North => "North",
            Direction::East => "East",
            Direction::South => "South",
            Direction::West => "West",
            Direction::Stay => "Stay",
        }
    }
}
