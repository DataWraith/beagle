use direction::Direction;

#[derive(Clone, Copy, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

impl Position {
    pub fn neighbor(&self, dir : Direction) -> Position {
        match dir {
            Direction::North => Position{x: self.x - 1, y: self.y},
            Direction::East  => Position{x: self.x, y: self.y + 1},
            Direction::South => Position{x: self.x + 1, y: self.y},
            Direction::West  => Position{x: self.x, y: self.y - 1},
            Direction::Stay  => Position{x: self.x, y: self.y},
        }
    }

    pub fn neighbors(&self) -> [Position; 4] {
        [Position{x: self.x - 1, y: self.y},
         Position{x: self.x, y: self.y + 1},
         Position{x: self.x + 1, y: self.y},
         Position{x: self.x, y: self.y - 1}]
    }

    pub fn manhattan_distance(&self, other : &Position) -> usize  {
        (self.x - other.x).abs() as usize + (self.y - other.y).abs() as usize
    }
}
