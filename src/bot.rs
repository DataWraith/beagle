use std::collections::HashMap;
use std::hash::{Hash, Hasher, SipHasher};

use time;
use rand;
use rand::Rng;

use state::State;

struct Node {
    scores: [f32; 4],
    visits: f32,
}

pub struct Bot {
    nodes: HashMap<u64, Node>,
}

impl Bot {
    pub fn new() -> Bot {
        Bot{
            nodes: HashMap::new()
        }
    }

    pub fn choose_move(&mut self, s : &State) -> &'static str {
        let end_time = time::get_time() + time::Duration::milliseconds(800);
        let mut hasher = SipHasher::new();
        s.hash(&mut hasher);
        let root_hash = hasher.finish();
		let mut count = 0;

        while time::get_time() < end_time {
            let result = self.mcts(&mut s.clone(), 30);
			
			let mut new_scores = [
				result[0] + self.nodes[&root_hash].scores[0],
				result[1] + self.nodes[&root_hash].scores[1],
				result[2] + self.nodes[&root_hash].scores[2],
				result[3] + self.nodes[&root_hash].scores[3]				
			];
			
			let new_visits = 1f32 + self.nodes[&root_hash].visits;
			
			self.nodes.insert(root_hash, Node{
				scores: new_scores,
				visits: new_visits,
			});
			
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
            let mv = rand::thread_rng().choose(&moves).unwrap();
            s.make_move(mv);
        }

        let mut result = [0f32, 0f32, 0f32, 0f32];
        result[0] = s.game.heroes[0].gold as f32 + s.game.heroes[0].life as f32;
		result[1] = s.game.heroes[1].gold as f32 + s.game.heroes[1].life as f32;
		result[2] = s.game.heroes[2].gold as f32 + s.game.heroes[2].life as f32;
		result[3] = s.game.heroes[3].gold as f32 + s.game.heroes[3].life as f32;
		result[0] /= 10100f32;
		result[1] /= 10100f32;
		result[2] /= 10100f32;		
		result[3] /= 10100f32;       

        result
    }

    pub fn mcts(&mut self, s : &mut State, depth : u8) -> [f32; 4	] {
		let mut result = [0f32, 0f32, 0f32, 0f32];
        let mut hasher = SipHasher::new();
        s.hash(&mut hasher);
        let node_hash = hasher.finish();

        if !self.nodes.contains_key(&node_hash) {           
            result = self.playout(s);
			self.nodes.insert(node_hash, Node{
                scores: result,
                visits: 1f32,
            })	;
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
			
            let sort_score = xavg + (2f32 * self.nodes[&node_hash].visits.ln() / visits).sqrt();

            if sort_score > max_score {
                max_score = sort_score;
                max_move = mv;
            }
        }

        s.make_move(max_move);
		if depth > 0 {
			let result = self.mcts(s, depth - 1);
		} else {
			let result = self.playout(s);
		}
		
		let new_scores = [
			result[0] + self.nodes[&node_hash].scores[0],
			result[1] + self.nodes[&node_hash].scores[1],
			result[2] + self.nodes[&node_hash].scores[2],
			result[3] + self.nodes[&node_hash].scores[3],
		];
		
		let new_visits = 1f32 + self.nodes[&node_hash].visits;
		
		self.nodes.insert(node_hash, Node{
			scores: new_scores,
			visits: new_visits,
		});
		
        result
    }
}
