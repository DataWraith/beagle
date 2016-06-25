use std::fmt;

#[derive(Clone, Copy, Deserialize, Debug, Eq, PartialEq)]
pub enum Tile {
    Wall,
    Air,
    Tavern,
    Mine(usize),
    Hero(usize),
}

impl Default for Tile {
    fn default() -> Tile {
        Tile::Wall
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Tile::Wall   => write!(f, "##"),
            Tile::Air    => write!(f, "  "),
            Tile::Tavern => write!(f, "[]"),
            Tile::Mine(0) => write!(f, "$-"),
            Tile::Mine(x) => write!(f, "${}", x),
            Tile::Hero(x) => write!(f, "@{}", x),
        }
    }
}
