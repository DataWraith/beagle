#[derive(Clone, Deserialize, Debug, Eq, PartialEq)]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

impl Position {
    pub fn neighbor(&self, dir : &str) -> Position {
        match dir {
            "North" => Position{x: self.x - 1, y: self.y},
            "East"  => Position{x: self.x, y: self.y + 1},
            "South" => Position{x: self.x + 1, y: self.y},
            "West"  => Position{x: self.x, y: self.y - 1},
            "Stay"  => Position{x: self.x, y: self.y},
            _ => panic!("Invalid direction: {}", dir),
        }
    }

    pub fn neighbors(&self) -> [Position; 4] {
        [Position{x: self.x - 1, y: self.y},
         Position{x: self.x, y: self.y + 1},
         Position{x: self.x + 1, y: self.y},
         Position{x: self.x, y: self.y - 1}]
    }

    pub fn manhattan_distance(&self, other : &Position) -> usize  {
        ((self.x - other.x).abs() as usize + (self.y - other.y).abs() as usize)
    }
}
