use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher, SipHasher};

use time;
use rand;
use rand::Rng;

use state::State;
use position::Position;
use tile::Tile;

struct Node {
    scores: [f32; 4],
    visits: f32,
}

pub struct Bot {
    initialized: bool,
    nodes: HashMap<u64, Box<Node>>,
	tavern_dist: HashMap<Position, u8>,
	mine_dist: HashMap<Position, Vec<(Position, u8, &'static str)>>,
}

impl Bot {
    pub fn new() -> Bot {
        Bot{
			initialized: false,
            nodes: HashMap::new(),
			tavern_dist: HashMap::new(),
			mine_dist: HashMap::new(),
        }
    }
	
	fn get_closest_tavern(&self, pos : &Position) -> (u8, &'static str) {
		let mut min_dist = 255u8;
		let mut min_dir = "Stay";
		
		for mv in ["North", "East", "South", "West"].iter() {
			let t = pos.neighbor(mv);
			if self.tavern_dist.contains_key(&t) {
				let dist = self.tavern_dist[&t];
				if dist < min_dist {
					min_dist = dist;
					min_dir = mv;
				}
			}
		}
		
		(min_dist, min_dir)
	}
	
	fn get_closest_mine(&mut self, pos : &Position, player_id : usize, s : &State) -> (u8, &'static str) {
		if !self.mine_dist.contains_key(pos) {
			let mut queue = VecDeque::new();
			let mut seen = HashSet::new();
			let mut result = Vec::new();
			
			seen.insert(*pos);
			queue.push_back((pos.neighbor("North"), 1, "North"));
			queue.push_back((pos.neighbor("East"), 1, "East"));
			queue.push_back((pos.neighbor("South"), 1, "South"));
			queue.push_back((pos.neighbor("West"), 1, "West"));
			
			while !queue.is_empty() {
				let (cur, dist, dir) = queue.pop_front().unwrap();
				
				match s.game.board.tile_at(&cur) {
					Tile::Mine(_) => {
						result.push((cur, dist, dir));
					}
					Tile::Air | Tile::Hero(_) => {
						for n in cur.neighbors().iter() {
							if !seen.contains(n) {
								queue.push_back((*n, dist + 1, dir));
								seen.insert(cur);
							}
						}
					},
					_ => (),
				}
			}
			
			self.mine_dist.insert(*pos, result);
		}
		
		for &(mpos, mdist, mdir) in self.mine_dist[pos].iter() {
			match s.game.board.tile_at(&mpos) {
				Tile::Mine(x) if x != player_id => return (mdist, mdir),
				_ => (),
			}
		}
		
		(0, "Stay")
	}
	
	fn initialize(&mut self, s : &State) {
		let mut queue = VecDeque::new();
		
		for x in 0..s.game.board.size {
			for y in 0..s.game.board.size {
				match s.game.board.tile_at(&Position{x: x, y: y}) {
					Tile::Tavern => {
						queue.push_back(Position{x: x, y: y});
						self.tavern_dist.insert(Position{x: x, y: y}, 0u8);
					},
					_ => (),
				}
			}
		}
		
		while !queue.is_empty() {
			let cur = queue.pop_front().unwrap();
			
			for n in cur.neighbors().iter() {
				if !self.tavern_dist.contains_key(n) {
					match s.game.board.tile_at(n) {
					Tile::Air | Tile::Hero(_) => {
							let dist = self.tavern_dist[&cur] + 1u8;
							self.tavern_dist.insert(*n, dist);
							queue.push_back(*n);
						}
						_ => (),
					}
				}
			}
		}
	
		self.initialized = true;
	}

    pub fn choose_move(&mut self, s : &State) -> &'static str {
        let end_time = time::get_time() + time::Duration::milliseconds(800);
		
		if !self.initialized {
			self.initialize(s);
		}
		
        let mut hasher = SipHasher::new();
        s.hash(&mut hasher);
        let root_hash = hasher.finish();
		let mut count = 0;

        while time::get_time() < end_time {
            let result = self.mcts(&mut s.clone(), 64);
			
			let new_scores = [
				result[0] + self.nodes[&root_hash].scores[0],
				result[1] + self.nodes[&root_hash].scores[1],
				result[2] + self.nodes[&root_hash].scores[2],
				result[3] + self.nodes[&root_hash].scores[3]				
			];
			
			let new_visits = 1f32 + self.nodes[&root_hash].visits;
			
			self.nodes.insert(root_hash, Box::new(Node{
				scores: new_scores,
				visits: new_visits,
			}));
			
			count += 1;
        }

		println!("Count: {}", count);
		
        let mut max_visits : f32 = 0f32;
        let mut best_dir : &'static str = "Stay";
		for mv in s.get_moves() {
			let mut st = s.clone();
			st.make_move(mv);
			let mut st_hasher = SipHasher::new();
			st.hash(&mut st_hasher);
			let st_hash = st_hasher.finish();
			
			if self.nodes.contains_key(&st_hash) {
				println!("Visits: {}", self.nodes[&st_hash].visits);
				if self.nodes[&st_hash].visits > max_visits {
					max_visits = self.nodes[&st_hash].visits;
					best_dir = mv;
				}
			}
		}
	
        best_dir
    }

    pub fn playout(&self, s : &mut State) -> [f32; 4] {
        while !s.get_moves().is_empty() {
			let moves = s.get_moves();
			let (_, tdir) = self.get_closest_tavern(&s.game.heroes[s.game.turn%4].pos);
			let (mdist, mdir) = self.get_closest_tavern(&s.game.heroes[s.game.turn%4].pos);
			let mut mv;
			
			if rand::thread_rng().next_f32() < 0.1 {
				mv = rand::thread_rng().choose(&moves).unwrap();
			} else {
				if s.game.heroes[s.game.turn % 4].life - mdist < 20 {
					mv = &tdir;
				} else {
					mv = &mdir;
				}
			}
            s.make_move(mv);
        }

        let mut result = [0f32, 0f32, 0f32, 0f32];
        result[0] = s.game.heroes[0].gold as f32 * 35f32 + s.game.heroes[0].life as f32;
		result[1] = s.game.heroes[1].gold as f32 * 35f32 + s.game.heroes[1].life as f32;
		result[2] = s.game.heroes[2].gold as f32 * 35f32 + s.game.heroes[2].life as f32;
		result[3] = s.game.heroes[3].gold as f32 * 35f32 + s.game.heroes[3].life as f32;
		result[0] /= 350100f32;
		result[1] /= 350100f32;
		result[2] /= 350100f32;		
		result[3] /= 350100f32;       

        result
    }

    pub fn mcts(&mut self, s : &mut State, depth : u8) -> [f32; 4	] {
		let mut result = [0f32, 0f32, 0f32, 0f32];
        let mut hasher = SipHasher::new();
        s.hash(&mut hasher);
        let node_hash = hasher.finish();

        if !self.nodes.contains_key(&node_hash) {           
            result = self.playout(s);
			self.nodes.insert(node_hash, Box::new(Node{
                scores: result,
                visits: 1f32,
            }))	;
            return result;
        }

        let mut max_score : f32 = 0f32;
        let mut max_move = "Stay";
        for mv in s.get_moves() {
            let mut st = s.clone();
            st.make_move(mv);
            let mut st_hasher = SipHasher::new();
            st.hash(&mut st_hasher);
            let st_hash = st_hasher.finish();
			
			let mut xavg = 0f32;
			let mut visits = 0f32;
			
			if self.nodes.contains_key(&st_hash) {
				xavg = self.nodes[&st_hash].scores[s.game.turn % 4];
				visits = self.nodes[&st_hash].visits;
			} 
			
			let mut sort_score : f32;
			
			if visits == 0f32 {
				sort_score = 10000f32 + rand::thread_rng().next_f32();
			} else {
			//let turns_left = (st.game.max_turns - st.game.turn) / 4;
			//let heuristic = (st.game.heroes[s.game.turn % 4].gold as f32 + turns_left as f32 * st.game.heroes[s.game.turn % 4].mine_count as f32) / 10000f32;
			
             sort_score = xavg + 1.4f32 *(self.nodes[&node_hash].visits.ln() / visits).sqrt(); // + heuristic / (1f32 + visits);
			}

            if sort_score > max_score {
                max_score = sort_score;
                max_move = mv;
            }
        }

        s.make_move(max_move);
		if depth > 0 {
			result = self.mcts(s, depth - 1);
		} else {
			result = self.playout(s);
		}
		
		let new_scores = [
			result[0] + self.nodes[&node_hash].scores[0],
			result[1] + self.nodes[&node_hash].scores[1],
			result[2] + self.nodes[&node_hash].scores[2],
			result[3] + self.nodes[&node_hash].scores[3],
		];
		
		let new_visits = 1f32 + self.nodes[&node_hash].visits;
		
		self.nodes.insert(node_hash, Box::new(Node{
			scores: new_scores,
			visits: new_visits,
		}));
		
        result
    }
}
