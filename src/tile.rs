use std::fmt;

#[derive(Clone, Copy, Deserialize, Debug, Eq, PartialEq, Hash)]
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

impl Tile {
	pub fn to_usize(&self) -> usize {
		match *self {
			Tile::Wall => 0,
			Tile::Air  => 1,
			Tile::Tavern => 2,
			Tile::Mine(x) => 3 + x as usize,
			Tile::Hero(x) => 7 + x as usize,
		}
	}
}