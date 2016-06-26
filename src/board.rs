use std::fmt;
use std::hash;

use tile::Tile;
use position::Position;

#[derive(Clone, Deserialize, Debug, Eq)]
pub struct Board {
    pub size: i8,
    #[serde(default)]
    board: Vec<Tile>,
    #[serde(default)]
    initialized: bool,
    tiles: String,
    #[serde(default)]
    pub mine_pos : Vec<Position>,
}

impl hash::Hash for Board {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.initialized.hash(state);
        for t in self.board.iter() {
            t.hash(state);
        }
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Board) -> bool {
        if self.size != other.size {
            return false;
        }

        for i in 0..((self.size as usize) * (self.size as usize)) {
            if self.board[i] != other.board[i] {
                return false;
            }
        }

        true
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for x in 0..self.size {
            for y in 0..self.size {
                write!(f, "{}", self.tile_at(&Position{x: x, y: y}));
            }
            write!(f, "\n");
        }
        Ok(())
    }
}

impl Board {
    pub fn initialize(&mut self) {
      self.board = vec![Tile::Wall; (self.size as usize * self.size as usize)];
      self.mine_pos = vec![];

      let b = self.tiles.as_bytes();

      for i in 0..((self.size as usize * self.size as usize)) {
          self.board[i] = match (b[2 * i], b[2 * i + 1]) {
              (35, 35) => Tile::Wall,
              (32, 32) => Tile::Air,
              (91, 93) => Tile::Tavern,
              (36, 45) => Tile::Mine(0),
              (36, 49) => Tile::Mine(1),
              (36, 50) => Tile::Mine(2),
              (36, 51) => Tile::Mine(3),
              (36, 52) => Tile::Mine(4),
              (64, 49) => Tile::Hero(1),
              (64, 50) => Tile::Hero(2),
              (64, 51) => Tile::Hero(3),
              (64, 52) => Tile::Hero(4),
              _ => panic!("Unprocessable tile found.")
          }
      }

      for x in 0..(self.size) {
          for y in 0..(self.size) {
              let idx = (self.size as usize) * (x as usize) + (y as usize);

             match  self.board[idx] {
                 Tile::Mine(_) => self.mine_pos.push(Position{x: x, y: y}),
                 _ => (),
             }
          }
      }

      self.initialized = true;
    }

    pub fn tile_at(&self, pos : &Position) -> Tile {
      if !self.initialized {
          panic!("tile_at called on uninitialized board")
      }

      if pos.x < 0 || pos.x >= self.size || pos.y < 0 || pos.y >= self.size {
          return Tile::Wall;
      }

      let idx = (pos.x as usize) * (self.size as usize) + (pos.y as usize);
      self.board[idx].clone()
    }

    pub fn put_tile(&mut self, pos : &Position, t : Tile) {
        let idx = (pos.x as usize) * (self.size as usize) + (pos.y as usize);
        self.board[idx] = t;
    }
}
