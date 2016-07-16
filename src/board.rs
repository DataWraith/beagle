use std::fmt;
use std::hash;
use std::hash::{Hash, Hasher};

use fnv::FnvHasher;

use std::collections::HashSet;

use tile::Tile;
use position::Position;
use zobrist::ZOBRIST;

#[derive(Clone, Deserialize, Debug, Eq)]
pub struct Board {
    pub size: i8,
    #[serde(default)]
    board: Vec<Tile>,
    #[serde(default)]
    initialized: bool,
    tiles: String,
    #[serde(default)]
    pub mine_pos: Vec<Position>,
    #[serde(default)]
    pub tavern_pos: Vec<Position>,
    #[serde(default)]
    pub hash: u64,
}

impl hash::Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Board) -> bool {
        let mut sh = FnvHasher::default();
        self.hash(&mut sh);
        let shash = sh.finish();

        let mut oh = FnvHasher::default();
        other.hash(&mut oh);
        let ohash = oh.finish();

        shash == ohash
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.initialized {
            for x in 0..self.size {
                for y in 0..self.size {
                    let res = write!(f, "{}", self.tile_at(&Position { x: x, y: y }));
                    if !res.is_ok() {
                        return res;
                    }
                }
                let res = write!(f, "\n");
                if !res.is_ok() {
                    return res;
                }
            }
        }
        Ok(())
    }
}

impl Board {
    pub fn initialize(&mut self) {
        self.board = vec![Tile::Wall; (self.size as usize * self.size as usize)];
        self.mine_pos = vec![];
        self.tavern_pos = Vec::with_capacity(4);
        self.hash = 0;

        {
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
                    _ => panic!("Unprocessable tile found."),
                };
                unsafe {
                    self.hash ^= ZOBRIST.keys[12 as usize * i as usize + self.board[i].to_usize()];
                }
            }


        }

        self.tiles = String::default();

        for x in 0..(self.size) {
            for y in 0..(self.size) {
                let idx = (self.size as usize) * (x as usize) + (y as usize);

                if let Tile::Mine(_) = self.board[idx] {
                    self.mine_pos.push(Position { x: x, y: y });
                } else if let Tile::Tavern = self.board[idx] {
                    self.tavern_pos.push(Position { x: x, y: y });
                }
            }
        }

        self.initialized = true;
    }

    pub fn tile_at(&self, pos: &Position) -> Tile {
        if !self.initialized {
            panic!("tile_at called on uninitialized board")
        }

        if pos.x < 0 || pos.x >= self.size || pos.y < 0 || pos.y >= self.size {
            return Tile::Wall;
        }

        let idx = (pos.x as usize) * (self.size as usize) + (pos.y as usize);
        self.board[idx]
    }

    pub fn put_tile(&mut self, pos: &Position, t: Tile) {
        let idx = (pos.x as usize) * (self.size as usize) + (pos.y as usize);

        unsafe {
            self.hash ^= ZOBRIST.keys[12 * idx + self.board[idx].to_usize()];
            self.board[idx] = t;
            self.hash ^= ZOBRIST.keys[12 * idx + self.board[idx].to_usize()];
        }
    }

    // Hadlock's shortest path algorithm
    pub fn shortest_path_length(&self,
                                start: &Position,
                                goal: &Position,
                                max_dist: u8)
                                -> Option<u8> {
        // Step 1
        let mut u = *start;
        let mut d = 0u8;
        let mut visited = HashSet::new();
        let mut pos_stack = Vec::new();
        let mut neg_stack = Vec::new();


        if start.manhattan_distance(goal) as u8 >= max_dist {
            return None;
        }

        // Step 2
        'outer: loop {
            if u == *goal {
                return Some(start.manhattan_distance(goal) as u8 + 2 * d);
            }

            visited.insert(u);

            let mut num_pos = 0;

            for v in &u.neighbors() {
                match self.tile_at(v) {
                    Tile::Wall | Tile::Hero(_) => (),
                    Tile::Tavern | Tile::Mine(_) => {
                        if v == goal {
                            pos_stack.push(v.clone());
                            num_pos += 1;
                        }
                    }
                    Tile::Air => {
                        if !visited.contains(v) {
                            let dist_diff: i8 = u.manhattan_distance(goal) as i8 -
                                                v.manhattan_distance(goal) as i8;
                            if dist_diff < 0 {
                                neg_stack.push(v.clone());
                            } else {
                                pos_stack.push(v.clone());
                                num_pos += 1;
                            }
                        }
                    }
                }
            }

            if num_pos > 0 {
                u = pos_stack.pop().unwrap().clone();
                continue 'outer;
            }

            // Step 3
            'step3: loop {
                while !pos_stack.is_empty() {
                    let v = pos_stack.pop().unwrap();
                    if visited.contains(&v) {
                        continue;
                    } else {
                        u = v.clone();
                        continue 'outer;
                    }
                }

                // Step 4
                if !neg_stack.is_empty() {
                    pos_stack = neg_stack.clone();
                    neg_stack = Vec::new();
                    d += 1;
                    if start.manhattan_distance(goal) as u8 + 2 * d >= max_dist {
                        return None;
                    }
                    continue 'step3;
                } else {
                    break;
                }
            }

            // Step 5
            return None;
        }
    }
}
