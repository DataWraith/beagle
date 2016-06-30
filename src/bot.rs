use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use fnv::FnvHasher;

use time;
use rand;
use rand::Rng;

use state::State;
use position::Position;
use tile::Tile;
use transposition_table::{Table, Entry};
use zobrist::ZobristTable;

pub struct Bot {
    initialized: bool,
	tt: Table,
    tavern_dist: HashMap<Position, u8>,
    mine_dist: HashMap<Position, Vec<(Position, u8, &'static str)>>,
}

impl Bot {
    pub fn new() -> Bot {
        Bot{
            initialized: false,
            tt: Table::new(1000000u64),
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

        (min_dist + 1, min_dir)
    }

    fn get_closest_mine(&mut self, pos : &Position, player_id : usize, s : &Box<State>) -> (u8, &'static str) {
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

    fn initialize(&mut self, s : &Box<State>) {
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

    fn eval(&mut self, s : &Box<State>) -> f32 {
        let turns_left = (s.game.max_turns - s.game.turn) / 4;
        let mut pred_gold = (s.hero.gold as usize + s.hero.mine_count as usize * turns_left as usize) + s.hero.life as usize / 10;
		let mut neg_gold = 0 as usize;
		
		for h in s.game.heroes.iter() {
			if h.name == s.hero.name {
				continue;
			}
			
			let hero_gold = h.gold as usize + (1 + h.mine_count as usize) * turns_left;
	
			neg_gold += hero_gold;			
		}
		
		for h in s.game.heroes.iter() {
			if h.name == s.hero.name {
				continue;
			}
			
			if h.pos.manhattan_distance(&s.hero.pos) == 1 {
				neg_gold += 25;
			}
		}
		
		
		
		let (mdist, mdir) = self.get_closest_mine(&s.hero.pos, s.hero.id, s);
        let mut delay = 0 as usize;
        if mdist > 0 {
            if s.hero.life < mdist || s.hero.life - mdist <= 20 {
                let (tdist, _) = self.get_closest_tavern(&s.hero.pos);
                delay += 2*tdist as usize;
            }
            delay += mdist as usize;
        } else {
            let (tdist, _) = self.get_closest_tavern(&s.hero.pos);
            delay += tdist as usize;
        }

        if delay < turns_left {
           pred_gold += 1 * (turns_left - delay);
        }
		
		
		(3f32 * pred_gold as f32 - neg_gold as f32)
		//(pred_gold as f32 - delay as f32)
    }

    fn brs(&mut self, s : &Box<State>, alphao : f32, betao : f32, depth : u8, end_time : time::Timespec, nodes : &mut u64) -> Option<f32> {
		let mut alpha = alphao;
		let mut beta = betao;
		let mut bmove = "Stay";
		let mut have_hash_move = false;
		let mut bscore = -1000000f32;
		let mut g : f32;
		let mut a : f32;
		let mut b : f32;
	
		let mut sh = FnvHasher::default();
		s.hash(&mut sh);
		let hash = sh.finish();
		let entry = self.tt.probe(hash);
		
		if entry.is_some() {
			let e = entry.unwrap();
			if e.turn >= depth as u16 {
			
			have_hash_move = true;
			bmove = e.mv;
			//bscore = e.lower;
			
			if e.lower >= beta {
				return Some(e.lower);
			}
			if e.upper <= alpha {
				return Some(e.upper);
			}
			
			if e.lower > alpha {
				alpha = e.lower;
			}
			
			if e.upper < beta {
				beta = e.upper;
			}}
		}
		
		if (*nodes < 10u64 || *nodes & 511u64 == 511) && time::get_time() > end_time {
			return None;
        }
		
		*nodes += 1;
		
        if depth == 0 || s.game.turn == s.game.max_turns {
            g = self.eval(&s);
        } else if s.game.turn % 4 == s.hero.id - 1 {		
			g = -1000000f32;
			a = alpha;
			
            let mut state = s.clone();
			
			// Try the hash move
			if have_hash_move {
				state.make_move(bmove);
				let v = self.brs(&state, a, beta, depth - 1, end_time, nodes);
				if v.is_none() {
					return None;
				}
				let score = v.unwrap();
				if score > bscore {
					bscore = score;
				}
				if score > g {
					g = score;
				}
				if g > a {
					a = g;
				}
			}
			
			let mut state = s.clone();
            for mv in state.get_moves().iter() {			
				if g >= beta {
					break;
				}
				
                state = s.clone();
                state.make_move(mv);
                let v = self.brs(&state, a, beta, depth - 1, end_time, nodes);
                if v.is_none() {
                    return None;
                }
				let score = v.unwrap();
				if score > bscore {
					bmove = mv;
					bscore = score;
				}
                if score > g {
                    g =  score;
                }
				if g > a {
					a = g;
				}
				
				
            }
        } else {
			g = 1000000f32;
			b = beta;
			
            let mut state = s.clone();
            // First player moves
            for mv in state.get_moves().iter() {
				if g <= alpha {
					break;
				}
				
				if *mv == "Stay" {
					continue;
				}
				
                state = s.clone();
                state.make_move(mv);
                state.make_move("Stay");
                state.make_move("Stay");
                let v = self.brs(&state, alpha, b, depth - 1, end_time, nodes);
				if v.is_none() {
                    return None;
                }
                if v.unwrap() < g {
                    g = v.unwrap();
                }
				if g < b {
					b = g;
				}
            }

            state = s.clone();
            // Second player moves
            state.make_move("Stay");
            for mv in state.get_moves().iter() {
				if g <= alpha {
					break;
				}
				
				if *mv == "Stay" {
					continue;
				}
				
                state = s.clone();
                state.make_move("Stay");
                state.make_move(mv);
                state.make_move("Stay");
                let v = self.brs(&state, alpha, b, depth - 1, end_time, nodes);
                if v.is_none() {
                    return None;
                }
                if v.unwrap() < g {
                    g = v.unwrap();
                }
				if g < b {
					b = g;
				}
            }

            state = s.clone();
            state.make_move("Stay");
            state.make_move("Stay");
            for mv in state.get_moves().iter() {
				if g <= alpha {
					break;
				}
				
				if *mv == "Stay" {
					continue;
				}
				
                state = s.clone();
                state.make_move("Stay");
                state.make_move("Stay");
                state.make_move(mv);
                let v = self.brs(&state, alpha, b, depth - 1, end_time, nodes);
                if v.is_none() {
                    return None;
                }
                if v.unwrap() < g {
                    g = v.unwrap();
                }
				if g < b {
					b = g;
				}
            }
        }
		
		let mut e = Entry::default();
		if g <= alpha {
			e.upper = g;
			e.mv = bmove;
		} 
		if g > alpha && g < beta {
			e.upper = g;
			e.lower = g;
			e.mv = bmove;
		} 
		if g >= beta {	
			e.lower = g;
			e.mv = bmove;
		}
		if e.lower == 0f32 {
			e.lower = -1000000f32;
		}
		if e.upper == 0f32 {
			e.upper = 1000000f32;
		}
		e.turn = depth as u16;
		e.hash = hash;
		e.age = s.game.turn as u16;
		
		self.tt.store(e);
		
		return Some(g);		
    }
	
	pub fn mtdf(&mut self, s : &Box<State>, firstguess : f32, depth : u8, mut num_nodes : &mut u64, end_time : time::Timespec) -> Option<f32> {
		let mut g = firstguess;
		let mut upper = 1000000f32;
		let mut lower = -1000000f32;
		let mut beta : f32;
		let mut direction = 1f32;
		let mut step_size = 1f32;
		loop {
		
		
			if g == lower {
				beta = g + step_size;
			} else {
				beta = g;
			}
			
			let val = self.brs(s, beta - step_size, beta, depth, end_time, &mut num_nodes);
			
			if val.is_none() {
				return None;
			}
			g = val.unwrap();
			
			if g < beta {
				if direction < 0f32 {
					direction = 1f32;
				} else {
				    direction += 1f32;
				}
				upper = g;
				step_size = direction;
				if step_size > 10f32 {
					step_size = 10f32;
				}
			} else {
				if direction > 0f32 {
					direction = -1f32;
				} else {
					direction -= 1f32;
				}
								
				lower = g;
				
				step_size = -direction;
				if step_size > 10f32 {
					step_size = 10f32;
				}
			}
			
			if lower >= upper {
				break;
			}
		}
		
		Some(g)
	}

    pub fn choose_move(&mut self, s : &Box<State>) -> &'static str {
        let end_time = time::get_time() + time::Duration::milliseconds(750);

        if !self.initialized {
            self.initialize(s);
			if time::get_time() + time::Duration::milliseconds(200) > end_time {
				return "Stay";
			}
        }
		
		let mut depth = 0u8;
		let mut num_nodes = 0u64;
        let mut firstguess = self.eval(s);
		
			let mut best_v = -1000000f32;
		let mut best_u = -1000000f32;
		let mut best_d = "Stay";
		let mut prev_b = "Stay";

        while time::get_time() < end_time && depth <= 32 {
            depth += 1;
			let v = self.mtdf(s, firstguess, depth, &mut num_nodes, end_time);
			if v.is_some() {
				firstguess = v.unwrap();
						
			let mut sh = FnvHasher::default();
			s.hash(&mut sh);
			let hash = sh.finish();
		
			let entry = self.tt.probe(hash);
			if entry.is_some() {
				let e = entry.unwrap();
				println!("{}: [{}, {}], {}", e.mv, e.lower, e.upper, e.turn);
				prev_b = best_d;
				best_d = e.mv;
//				if e.upper > best_v || (e.upper == best_v && e.lower > best_u){
					
//					best_d = mv;
//					best_u = e.lower;
//					best_v = e.upper;
//					
				//}
//			}
		//}

			}
        }
	
	
		
//		for mv in s.get_moves().iter() {
//			let mut snext = s.clone();
//			snext.make_move(mv);
			
			//let mut sh = FnvHasher::default();
			//s.hash(&mut sh);
			//let hash = sh.finish();
		
			//let entry = self.tt.probe(hash);
			//if entry.is_some() {
				//let e = entry.unwrap();
				//println!("{}: [{}, {}], {}", e.mv, e.lower, e.upper, e.turn);
				//best_d = e.mv;
//				if e.upper > best_v || (e.upper == best_v && e.lower > best_u){
//					best_d = mv;
//					best_u = e.lower;
//					best_v = e.upper;
//					
				}
//			}
		//}

      
        println!("{}, {} - {} - {} - {}, nodes: {}", depth, best_d, firstguess, end_time - time::get_time(), s.hero.life, num_nodes);

        return best_d;

    }
}
