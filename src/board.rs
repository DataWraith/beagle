use std::fmt;
use std::hash;
use std::hash::{Hash, Hasher};
use std::collections::VecDeque;
use std::cmp;

use fnv::FnvHasher;

use std::collections::HashSet;

use tile::Tile;
use position::Position;
use zobrist::ZOBRIST;
use direction::Direction;

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

    #[serde(default)]
    pathcache: Vec<Vec<u8>>,
    #[serde(default)]
    minecache: Vec<(u8, Position)>,
    #[serde(default)]
    pub max_dist: u8,
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
        Ok(())}
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

        self.pathcache = vec![Vec::new(); self.size as usize * self.size as usize];
	self.minecache = vec![(0u8, Position{x:0, y:0}); self.size as usize * self.size as usize];
        self.initialized = true;
    }

    fn position_idx(&self, pos: &Position) -> usize {
        return pos.x as usize * (self.size as usize) + pos.y as usize;
    }

    fn bfs(&mut self, start: &Position) -> Vec<u8>{
        let mut dist = vec![255; (self.size as usize) * (self.size as usize)];
        dist[self.position_idx(start)] = 0;

        match self.tile_at(start) {
            Tile::Tavern | Tile::Mine(_) | Tile::Wall => { return dist },
            _ => {},
        }

        let mut q = VecDeque::new();
        q.push_back(*start);

        while !q.is_empty() {
            let cur = q.pop_front().unwrap();
            let cur_idx = self.position_idx(&cur);
            let nb = cur.neighbors();

            for v in &nb {
                match self.tile_at(v) {
                    Tile::Air | Tile::Hero(_) => {
                        let cidx = self.position_idx(v);
                        if dist[cidx] == 255 {
			    self.max_dist = cmp::max(self.max_dist, dist[cur_idx] + 1);
                            dist[cidx] = dist[cur_idx] + 1;
                            q.push_back(v.clone());
                        }
                    },

                    Tile::Tavern | Tile::Mine(_) => {
                        let cidx = self.position_idx(v);
                        if dist[cidx] == 255 {
			    self.max_dist = cmp::max(self.max_dist, dist[cur_idx] + 1);
                            dist[cidx] = dist[cur_idx] + 1;
                        }
                    },

                    _ => {},
                }
            }
        }

        return dist
    }

    pub fn direction_to(&mut self, from: &Position, to: &Position) -> Direction {
        let mut min_dist = 255u8;
        let mut min_dir  = Direction::Stay;

        for dir in &[Direction::North, Direction::East, Direction::South, Direction::West] {
            let n = from.neighbor(*dir);

	    if n.x < 0 || n.y < 0 || n.x >= self.size || n.y >= self.size {
	      continue;
	    }

            let dist = self.shortest_path_length(&n, to);

            if dist < min_dist {
                min_dist = dist;
                min_dir = *dir;
            }
        }

        min_dir
    }

    pub fn get_closest_tavern(&mut self, pos: &Position) -> (u8, Position) {
        let mut min_dist = 255;
        let mut resultpos = Position{x: 0, y: 0};

        for tpos in &self.tavern_pos.clone() {
            let new_d = self.shortest_path_length(&pos, tpos);
            if min_dist > new_d {
                resultpos = *tpos;
                min_dist = new_d;
            }
        }

        (min_dist, resultpos)
    }

    pub fn get_closest_mine(&mut self, pos: &Position, player_id: usize) -> (u8, Option<Position>) {
        let start_idx = self.position_idx(pos);

        if start_idx < 0 || start_idx >= (self.size as usize) * (self.size as usize) {
            return (255, None);
        }

        if self.minecache[start_idx].0 == 0u8 {
            let mut min_dist = 255u8;
            let mut mpos = Position{x: 0, y:0}; 

            for mp in &self.mine_pos.clone() {
                let d = self.shortest_path_length(pos, &mp);
                if d < min_dist {
                    min_dist = d;
                    mpos = *mp;
                }
            }

            self.minecache[start_idx] = (min_dist, mpos);
        }

        if let Tile::Mine(x) = self.tile_at(&self.minecache[start_idx].1) {
		        if x != player_id {
			          return (self.minecache[start_idx].0, Some(self.minecache[start_idx].1));
		        }
	      }

        let mut min_dist = 255u8;
        let mut mpos = None;

        for mp in &self.mine_pos.clone() {
            let tile = self.tile_at(&mp);

            match tile {
                Tile::Mine(x) if x != player_id => {
                    let new_d = self.shortest_path_length(pos, &mp);
                    if min_dist > new_d {
                        min_dist = new_d;
                        mpos = Some(*mp)
                    }
                },

                _ => {},
            }

        }

        (min_dist, mpos)
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

    pub fn shortest_path_length(&mut self, start: &Position, goal: &Position) -> u8 {
        if !self.initialized {
            panic!("shortest_path_length called on uninitialized board")
        }

        let start_idx = self.position_idx(start);
        let goal_idx = self.position_idx(goal);

        if start_idx < 0 || start_idx >= (self.size as usize) * (self.size as usize) {
            return 255
        }

        if goal_idx < 0 || goal_idx >= (self.size as usize) * (self.size as usize) {
		        return 255
	      }

        if !self.pathcache[start_idx].is_empty() {
            return self.pathcache[start_idx][goal_idx];
        }

        if !self.pathcache[goal_idx].is_empty() {
            return self.pathcache[goal_idx][start_idx];
        }

        let tree = self.bfs(start);
        self.pathcache[start_idx] = tree;
        self.pathcache[start_idx][goal_idx]
    }
}
