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
use lru::LRU;

pub struct Bot {
    initialized: bool,
    threat_list: [u8; 4],
    max_history: LRU<(Position, Direction)>,
    min_history: LRU<(u8, Position, Direction)>,
    tt: Table,
}

impl Bot {
    pub fn new() -> Bot {
        Bot {
            initialized: false,
            threat_list: [1, 2, 3, 0],
            tt: Table::new(10000000u64),
            max_history: LRU::<(Position, Direction)>::new((Position{x: -1, y: -1}, Direction::Stay)),
            min_history: LRU::<(u8, Position, Direction)>::new((4, Position{x: -1, y: -1}, Direction::Stay)),
        }
    }

    fn initialize(&mut self, s: &State) {
        self.initialized = true;
    }

    fn eval(&mut self, s: &mut State) -> i32 {
        let turns_left = (s.game.max_turns - s.game.turn) / 4;
        let mut pred_score = [0f64, 0f64, 0f64, 0f64, 0f64];
        let mut rank_adj = [0f64, 0f64, 0f64, 0f64, 0f64];

        let mut eval = 0.0;

        for h in &s.game.heroes {
            pred_score[h.id] = (h.gold as f64 + (h.mine_count as usize * turns_left) as f64) + (h.life as f64 / 20f64);

            if h.name != s.hero.name {
                let edist = s.game.board.shortest_path_length(&s.hero.pos, &h.pos);

                if edist < 6 && edist != 3 && h.life / 20 <= s.hero.life / 20 {
                    eval += 1.0;
                } else if edist < 6 && edist != 3 && h.life / 20 > s.hero.life / 20 {
                    eval -= 1.0;
                } else if edist == 3 && h.life / 20 + 1 >= s.hero.life / 20 {
                    eval -= 1.0;
                }
            }
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

        let (mdist, mpos) = s.game.board.get_closest_mine(&s.hero.pos, s.hero.id);
        let delay;
        if mdist < 255 && (s.hero.life < mdist || s.hero.life - mdist <= 20) {
            let (tdist, tpos) = s.game.board.get_closest_tavern(&s.hero.pos);
            let (mdist2, mpos2) = s.game.board.get_closest_mine(&tpos, s.hero.id);
            delay = 2 + (tdist + mdist2) as usize;
        } else if mdist < 255 {
            delay = mdist as usize;
        } else {
            delay = turns_left;
        }

        if delay < turns_left {
            pred_score[s.hero.id] += (turns_left - delay) as f64;
        }

        for h in &s.game.heroes {
            if h.name == s.hero.name {
                eval += pred_score[h.id];
                eval += rank_adj[h.id] * 10000.0;
            } else {
                eval -= pred_score[h.id];
            }
        }

        (eval as i32)
    }

    fn generate_moves(&mut self, s: &mut State) -> Vec<Move> {
        let mut result = Vec::with_capacity(12);

        // MAX node
        if s.game.heroes[s.game.turn % 4].id == s.hero.id {
            for dir in &s.get_moves() {
                result.push(Move {
                    player: 0,
                    directions: [*dir, Direction::Stay, Direction::Stay, Direction::Stay],
                });
            }
        } else {
            // MIN node

            // First player
            for dir in &s.get_moves() {
                if *dir != Direction::Stay {
                    result.push(Move {
                        player: 1,
                        directions: [Direction::Stay, *dir, Direction::Stay, Direction::Stay],
                    });
                }
            }

            // Second player
            let mut umi = s.make_move(Direction::Stay);
            for dir in &s.get_moves() {
                if *dir != Direction::Stay {
                    result.push(Move {
                        player: 2,
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
                        player: 3,
                        directions: [Direction::Stay, Direction::Stay, Direction::Stay, *dir],
                    });
                }
            }
            s.unmake_move(umi2);
            s.unmake_move(umi);

            result.push(Move {
                player: 1,
                directions: [Direction::Stay, Direction::Stay, Direction::Stay, Direction::Stay],
            });
        }

        result
    }

    fn pick_next_move(&mut self, depth: u8, hm: &Move, moves: &mut Vec<Move>, s: &State) -> Move {
        if moves.is_empty() {
            return Move::default();
        }

        let mut best_score = 0;
        let mut best_idx = 0;
        for (i, mv) in moves.iter().enumerate() {
            let mut score = 0;

            if mv == hm && *hm != Move::default() {
                best_idx = i;
                break
            } else if (*mv).player == self.threat_list[0] {
                score = 5;
            } else if (*mv).player == self.threat_list[1] {
                score = 3;
            } else if (*mv).player == self.threat_list[2] {
                score = 1;
            }

            if (*mv).player == 0 {
                let hist_score = self.max_history.query((s.hero.pos, mv.directions[0]));
                if hist_score != 255 {
                    score += hist_score;
                }
            } else {
                let hist_score = self.min_history.query(((*mv).player, s.game.heroes[(s.hero.id + (*mv).player as usize - 1)%4].pos, (*mv).directions[(*mv).player as usize]));
                if hist_score != 255 {
                    score += hist_score;
                }
            }

            if score > best_score {
                best_score = score;
                best_idx = i;
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
            if e.depth >= s.game.turn as u16 + depth as u16 {

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

                let curmove = self.pick_next_move(depth, &bmove, &mut moves, s);
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

                    self.max_history.insert((s.hero.pos, bmove.directions[0]));
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

                let curmove = self.pick_next_move(depth, &bmove, &mut moves, s);
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

                    self.min_history.insert((bmove.player, s.game.heroes[(s.hero.id - 1 + bmove.player as usize) % 4].pos, bmove.directions[0]));
                }
                if score < g {
                    g = score;
                }
                if g < b {
                    b = g;
                }
            }
        }

        if bmove.player != 0 {
            if self.threat_list[0] != bmove.player {
                if self.threat_list[1] != bmove.player {
                    if self.threat_list[2] != bmove.player {
                        self.threat_list[3] = self.threat_list[2];
                        self.threat_list[2] = self.threat_list[1];
                        self.threat_list[1] = self.threat_list[0];
                        self.threat_list[0] = bmove.player;
                    } else {
                        self.threat_list[2] = self.threat_list[1];
                        self.threat_list[1] = self.threat_list[0];
                        self.threat_list[0] = bmove.player;
                    }
                } else {
                    self.threat_list[1] = self.threat_list[0];
                    self.threat_list[0] = bmove.player;
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
        }

        e.depth = s.game.turn as u16 + depth as u16;
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
        let end_time = time::get_time() + time::Duration::milliseconds(800);

        if !self.initialized {
            self.initialize(s);
            if time::get_time() + time::Duration::milliseconds(200) > end_time {
                return Direction::Stay;
            }
        }

        // Clear history
        self.max_history = LRU::new((Position{x: -1, y: -1}, Direction::Stay));
        self.min_history = LRU::new((4u8, Position{x: -1, y: -1}, Direction::Stay));

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
                             e.depth - s.game.turn as u16);
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
