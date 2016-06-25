use tile::Tile;
use position::Position;

#[derive(Deserialize, Debug)]
pub struct Board {
    pub size: i8,
    #[serde(default)]
    board: Vec<Tile>,
    #[serde(default)]
    initialized: bool,
    tiles: String,
    pub mine_pos : Vec<Position>,
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

      for x in 1..(self.size) {
          for y in 1..(self.size) {
              let idx = (self.size as usize) * (y as usize) + (x as usize);

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

      let idx = pos.y * self.size + pos.x;
      self.board[idx as usize].clone()
    }

    pub fn put_tile(&mut self, pos : &Position, t : Tile) {
        let idx = (pos.y as usize) * (self.size as usize) + (pos.x as usize);
        self.board[idx] = t;
    }
}
