use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use fnv::FnvHasher;

use time;

use state::State;
use direction::Direction;
use mv::Move;
use position::Position;
use tile::Tile;
use transposition_table::{Table, Entry};

pub struct Bot {
    initialized: bool,
    killer1: [Move; 33],
    killer2: [Move; 33],
    tt: Table,
}

impl Bot {
    pub fn new() -> Bot {
        Bot {
            initialized: false,
            tt: Table::new(1000000u64),
            killer1: [Move::default(); 33],
            killer2: [Move::default(); 33],
        }
    }

    fn get_closest_tavern_dist(&mut self, s: &State, pos: &Position) -> u8 {
            let mut min_dist = 255;

            for tpos in &s.game.board.tavern_pos {
                let new_d = s.game.board.shortest_path_length(pos, tpos, min_dist);
                if new_d.is_some() {
                    min_dist = new_d.unwrap();
                }
            }

            return min_dist as u8;
    }

    fn get_closest_mine_dist(&mut self, pos: &Position, player_id: usize, s: &State) -> u8 {
        {
            let mut dist = vec![255; (s.game.board.size as usize * s.game.board.size as usize)];

            let mut q = VecDeque::new();
            q.push_back(*pos);

            let idx = (s.game.board.size as usize) * (pos.x as usize) + (pos.y as usize);
            dist[idx] = 0;

            while !q.is_empty() {
                let cur = q.pop_front().unwrap();
                let cur_idx = (s.game.board.size as usize) * (cur.x as usize) + (cur.y as usize);
                let nb = cur.neighbors();

                for v in &(nb) {
                    match s.game.board.tile_at(v) {
                        Tile::Mine(x) if x != player_id => {
                            return dist[cur_idx] + 1
                        }

                        Tile::Air => {
                            let child_idx = (s.game.board.size as usize) * (v.x as usize) + (v.y as usize);
                            if dist[child_idx] == 255 {
                                let child_dist = dist[cur_idx] + 1;
                                dist[child_idx] = child_dist;
                                q.push_back(v.clone())
                            }
                        }

                        _ => (),
                    }
                }
            }
        }
        0
    }


    fn initialize(&mut self, s: &State) {
        self.initialized = true;
    }

    fn eval(&mut self, s: &State) -> i32 {
        let turns_left = (s.game.max_turns - s.game.turn) / 4;
        let mut pred_score = [0f64, 0f64, 0f64, 0f64, 0f64];
        let mut rank_adj = [0f64, 0f64, 0f64, 0f64, 0f64];

        for h in &s.game.heroes {
            pred_score[h.id] = 10.0 *
                               (h.gold as f64 + (h.mine_count as usize * turns_left) as f64) +
                               h.life as f64;
        }

        for h in &s.game.heroes {
            for enemy in &s.game.heroes {
                if h.name == enemy.name {
                    continue;
                }

                let q_self = f64::powf(10.0, h.elo as f64 / 400.0);
                let q_enemy = f64::powf(10.0, enemy.elo as f64 / 400.0);
                let expected_self = q_self / (q_self + q_enemy);
                let mut actual = 1.0;
                if pred_score[h.id] < pred_score[enemy.id] {
                    actual = 0.0;
                } else if pred_score[h.id] == pred_score[enemy.id] {
                    actual = 0.5;
                }
                rank_adj[h.id] += 16.0 * (actual - expected_self);
            }
        }

        let mdist = self.get_closest_mine_dist(&s.hero.pos, s.hero.id, s);
        let mut delay = 0 as usize;
        if s.hero.life < mdist || s.hero.life - mdist <= 20 {
            let tdist = self.get_closest_tavern_dist(s, &s.hero.pos);
            delay += 2 * tdist as usize;
        }
        delay += mdist as usize;

        if delay < turns_left {
            rank_adj[s.hero.id] += (turns_left - delay) as f64;
        }

        let mut eval = 0.0;
        for h in &s.game.heroes {
            if h.name == s.hero.name {
                eval += pred_score[h.id];
            } else {
                eval -= pred_score[h.id];
            }
        }

        eval += rank_adj[s.hero.id] * 10.0;

        (eval as i32)
    }

    fn generate_moves(&mut self, s: &mut State) -> Vec<Move> {
        let mut result = Vec::with_capacity(12);

        // MAX node
        if s.game.heroes[s.game.turn % 4].id == s.hero.id {
            for dir in &s.get_moves() {
                result.push(Move {
                    directions: [*dir, Direction::Stay, Direction::Stay, Direction::Stay],
                });
            }
        } else {
            // MIN node

            // First player
            for dir in &s.get_moves() {
                if *dir != Direction::Stay {
                    result.push(Move {
                        directions: [Direction::Stay, *dir, Direction::Stay, Direction::Stay],
                    });
                }
            }

            // Second player
            let mut umi = s.make_move(Direction::Stay);
            for dir in &s.get_moves() {
                if *dir != Direction::Stay {
                    result.push(Move {
                        directions: [Direction::Stay, Direction::Stay, *dir, Direction::Stay],
                    });
                }
            }
            s.unmake_move(umi);

            // Third player
            umi = s.make_move(Direction::Stay);
            let umi2 = s.make_move(Direction::Stay);
            for dir in &s.get_moves() {
                if *dir != Direction::Stay {
                    result.push(Move {
                        directions: [Direction::Stay, Direction::Stay, Direction::Stay, *dir],
                    });
                }
            }
            s.unmake_move(umi2);
            s.unmake_move(umi);
        }

        result
    }

    fn pick_next_move(&self, depth: u8, hm: &Move, moves: &mut Vec<Move>) -> Move {
        if moves.is_empty() {
            return Move::default();
        }

        let mut best_score = 0;
        let mut best_idx = 0;
        for (i, mv) in moves.iter().enumerate() {
            let mut score = 1;

            if mv == hm && *hm != Move::default() {
                score = 1000;
            } else if *mv == self.killer1[depth as usize] || *mv == self.killer2[depth as usize] {
                score = 100;
            } else if *mv != Move::default() {
                score = 10
            }

            if score > best_score {
                best_score = score;
                best_idx = i;
                if best_score == 1000 {
                    break;
                }
            }
        }

        moves.swap_remove(best_idx)
    }

    fn brs(&mut self,
           s: &mut State,
           alphao: i32,
           betao: i32,
           depth: u8,
           end_time: time::Timespec,
           nodes: &mut u64)
           -> Option<i32> {
        let mut alpha = alphao;
        let mut beta = betao;
        let mut bmove = Move::default();
        let mut g: i32;
        let mut a: i32;
        let mut b: i32;

        let mut sh = FnvHasher::default();
        s.hash(&mut sh);
        let hash = sh.finish();
        let entry = self.tt.probe(hash);

        if entry.is_some() {
            let e = entry.unwrap();
            if e.depth >= depth as u16 {

                bmove = e.mv;
                // bscore = e.lower;

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
                }
            }
        }

        *nodes += 1;

        if (*nodes < 10u64 || *nodes & 1023u64 == 1023u64) && time::get_time() > end_time {
            return None;
        }

        if depth == 0 || s.game.turn > s.game.max_turns - 4 {
            g = self.eval(s);
        } else if s.game.turn % 4 == s.hero.id - 1 {
            let mut bscore = i32::min_value();
            g = i32::min_value();
            a = alpha;

            let mut moves = self.generate_moves(s);

            while !moves.is_empty() {
                if g >= beta {
                    break;
                }

                let curmove = self.pick_next_move(depth, &bmove, &mut moves);
                let umi = s.make_move(curmove.directions[0]);
                let v = self.brs(s, a, beta, depth - 1, end_time, nodes);
                s.unmake_move(umi);

                if v.is_none() {
                    return None;
                }

                let score = v.unwrap();
                if score > bscore {
                    bmove = curmove;
                    bscore = score;
                }
                if score > g {
                    g = score;
                }
                if g > a {
                    a = g
                }
            }
        } else {
            let mut bscore = i32::max_value();
            g = i32::max_value();
            b = beta;

            let mut moves = self.generate_moves(s);

            while !moves.is_empty() {
                if g <= alpha {
                    break;
                }

                let curmove = self.pick_next_move(depth, &bmove, &mut moves);
                let umi1 = s.make_move(curmove.directions[1]);
                let umi2 = s.make_move(curmove.directions[2]);
                let umi3 = s.make_move(curmove.directions[3]);

                let v = self.brs(s, alpha, b, depth - 1, end_time, nodes);

                s.unmake_move(umi3);
                s.unmake_move(umi2);
                s.unmake_move(umi1);

                if v.is_none() {
                    return None;
                }
                let score = v.unwrap();
                if score < bscore {
                    bmove = curmove;
                    bscore = score;
                }
                if score < g {
                    g = score;
                }
                if g < b {
                    b = g;
                }
            }
        }

        let mut e = Entry::default();
        if g <= alpha {
            e.upper = g;
            e.lower = i32::min_value();
            e.mv = bmove;
        } else if g > alpha && g < beta {
            e.upper = g;
            e.lower = g;
            e.mv = bmove;
        } else if g >= beta {
            e.lower = g;
            e.upper = i32::max_value();
            e.mv = bmove;

            if self.killer1[depth as usize] != bmove && self.killer2[depth as usize] != bmove {
                self.killer1[depth as usize] = self.killer2[depth as usize];
                self.killer2[depth as usize] = bmove;
            }
        }

        e.depth = depth as u16;
        e.hash = hash;
        e.age = s.game.turn as u16;

        self.tt.store(e);

        Some(g)
    }

    pub fn mtdf(&mut self,
                s: &mut State,
                firstguess: i32,
                depth: u8,
                mut num_nodes: &mut u64,
                end_time: time::Timespec)
                -> Option<i32> {
        let mut f = firstguess;
        let mut upper = i32::max_value();
        let mut lower = i32::min_value();
        let step_size = 25i32;

        while upper == i32::max_value() || lower == i32::min_value() {
            let val = self.brs(s, f - 1, f, depth, end_time, &mut num_nodes);
            if val.is_none() {
                return None;
            }

            let g = val.unwrap();

            if g < f {
                upper = g
            } else {
                lower = g
            }

            if upper == g {
                f = g - step_size;
            } else {
                f = g + step_size;
            }
        }

        if lower == upper {
            return Some(lower);
        }

        self.brs(s, lower, upper, depth, end_time, &mut num_nodes)
    }

    pub fn choose_move(&mut self, s: &mut State) -> Direction {
        let end_time = time::get_time() + time::Duration::milliseconds(750);

        if !self.initialized {
            self.initialize(s);
            if time::get_time() + time::Duration::milliseconds(200) > end_time {
                return Direction::Stay;
            }
        }

        // Clear killers
        for i in 0..33 {
            self.killer1[i] = Move::default();
            self.killer2[i] = Move::default();
        }

        let mut depth = 0u8;
        let mut num_nodes = 0u64;
        let mut firstguess = self.eval(s);

        let mut best_d = Direction::Stay;
        let mut prev_b = Direction::Stay;

        while time::get_time() < end_time && depth < 32 {
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
                    println!("{}: [{}, {}], {}",
                             e.mv.directions[0],
                             e.lower,
                             e.upper,
                             e.depth);
                    prev_b = best_d;
                    best_d = e.mv.directions[0];
                }
            }
        }

        println!("{}, {} - {} - {} - {}, nodes: {}",
                 depth,
                 prev_b,
                 firstguess,
                 end_time - time::get_time(),
                 s.hero.life,
                 num_nodes);

        prev_b
    }
}
