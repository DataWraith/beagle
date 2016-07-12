use std::collections::{HashMap, HashSet, VecDeque};
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
    tavern_dist: HashMap<Position, u8>,
    mine_dist: HashMap<Position, Vec<(Position, u8, Direction)>>,
}

impl Bot {
    pub fn new() -> Bot {
        Bot {
            initialized: false,
            tt: Table::new(1000000u64),
            tavern_dist: HashMap::new(),
            killer1: [Move::default(); 33],
            killer2: [Move::default(); 33],
            mine_dist: HashMap::new(),
        }
    }

    fn get_closest_tavern(&self, pos: &Position) -> (u8, Direction) {
        let mut min_dist = 255u8;
        let mut min_dir = Direction::Stay;

        for mv in &[Direction::North, Direction::East, Direction::South, Direction::West] {
            let t = pos.neighbor(*mv);
            if self.tavern_dist.contains_key(&t) {
                let dist = self.tavern_dist[&t];
                if dist < min_dist {
                    min_dist = dist;
                    min_dir = *mv;
                }
            }
        }

        (min_dist + 1, min_dir)
    }

    fn get_closest_mine(&mut self,
                        pos: &Position,
                        player_id: usize,
                        s: &Box<State>)
                        -> (u8, Direction) {
        if !self.mine_dist.contains_key(pos) {
            let mut queue = VecDeque::new();
            let mut seen = HashSet::new();
            let mut result = Vec::new();

            seen.insert(*pos);
            queue.push_back((pos.neighbor(Direction::North), 1, Direction::North));
            queue.push_back((pos.neighbor(Direction::East), 1, Direction::East));
            queue.push_back((pos.neighbor(Direction::South), 1, Direction::South));
            queue.push_back((pos.neighbor(Direction::West), 1, Direction::West));

            while !queue.is_empty() {
                let (cur, dist, dir) = queue.pop_front().unwrap();

                match s.game.board.tile_at(&cur) {
                    Tile::Mine(_) => {
                        result.push((cur, dist, dir));
                    }
                    Tile::Air | Tile::Hero(_) => {
                        for n in &cur.neighbors() {
                            if !seen.contains(n) {
                                queue.push_back((*n, dist + 1, dir));
                                seen.insert(cur);
                            }
                        }
                    }
                    _ => (),
                }
            }

            self.mine_dist.insert(*pos, result);
        }

        for &(mpos, mdist, mdir) in &self.mine_dist[pos] {
            match s.game.board.tile_at(&mpos) {
                Tile::Mine(x) if x != player_id => return (mdist, mdir),
                _ => (),
            }
        }

        (0, Direction::Stay)
    }

    fn initialize(&mut self, s: &Box<State>) {
        let mut queue = VecDeque::new();

        for x in 0..s.game.board.size {
            for y in 0..s.game.board.size {
                if let Tile::Tavern = s.game.board.tile_at(&Position { x: x, y: y }) {
                    queue.push_back(Position { x: x, y: y });
                    self.tavern_dist.insert(Position { x: x, y: y }, 0u8);
                }
            }
        }

        while !queue.is_empty() {
            let cur = queue.pop_front().unwrap();

            for n in &cur.neighbors() {
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

    fn eval(&mut self, s: &Box<State>) -> i32 {
        let turns_left = (s.game.max_turns - s.game.turn) / 4;
        let mut pred_gold = (s.hero.gold as usize +
                             s.hero.mine_count as usize * turns_left as usize) +
                            s.hero.life as usize / 10;
        let mut neg_gold = 0 as usize;

        for h in &s.game.heroes {
            if h.name == s.hero.name {
                continue;
            }

            let hero_gold = h.gold as usize + (1 + h.mine_count as usize) * turns_left;

            neg_gold += hero_gold;
        }

        for h in &s.game.heroes {
            if h.name == s.hero.name {
                continue;
            }

            if h.pos.manhattan_distance(&s.hero.pos) == 1 {
                neg_gold += 25;
            }
        }



        let (mdist, _) = self.get_closest_mine(&s.hero.pos, s.hero.id, s);
        let mut delay = 0 as usize;
        if mdist > 0 {
            if s.hero.life < mdist || s.hero.life - mdist <= 20 {
                let (tdist, _) = self.get_closest_tavern(&s.hero.pos);
                delay += 2 * tdist as usize;
            }
            delay += mdist as usize;
        } else {
            let (tdist, _) = self.get_closest_tavern(&s.hero.pos);
            delay += tdist as usize;
        }

        if delay < turns_left {
            pred_gold += turns_left - delay;
        }


        (3i32 * pred_gold as i32 - neg_gold as i32)
    }

    fn generate_moves(&mut self, s: &Box<State>) -> Vec<Move> {
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
            let mut state = s.clone();
            for dir in &s.get_moves() {
                if *dir != Direction::Stay {
                    result.push(Move {
                        directions: [Direction::Stay, *dir, Direction::Stay, Direction::Stay],
                    });
                }
            }

            // Second player
            state.make_move(Direction::Stay);
            for dir in &s.get_moves() {
                if *dir != Direction::Stay {
                    result.push(Move {
                        directions: [Direction::Stay, Direction::Stay, *dir, Direction::Stay],
                    });
                }
            }

            // Third player
            state = s.clone();
            state.make_move(Direction::Stay);
            state.make_move(Direction::Stay);
            for dir in &s.get_moves() {
                if *dir != Direction::Stay {
                    result.push(Move {
                        directions: [Direction::Stay, Direction::Stay, Direction::Stay, *dir],
                    });
                }
            }
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
           s: &Box<State>,
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

        if (*nodes < 10u64 || *nodes & 1023u64 == 1023u64) && time::get_time() > end_time {
            return None;
        }

        *nodes += 1;

        if depth == 0 || s.game.turn == s.game.max_turns {
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

                let mut state = s.clone();
                let curmove = self.pick_next_move(depth, &bmove, &mut moves);
                state.make_move(curmove.directions[0]);
                let v = self.brs(&state, a, beta, depth - 1, end_time, nodes);
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

                let mut state = s.clone();
                let curmove = self.pick_next_move(depth, &bmove, &mut moves);
                for i in 1..4 {
                    state.make_move(curmove.directions[i]);
                }

                let v = self.brs(&state, alpha, b, depth - 1, end_time, nodes);
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
        }
        if g > alpha && g < beta {
            e.upper = g;
            e.lower = g;
            e.mv = bmove;
        }
        if g >= beta {
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
                s: &Box<State>,
                firstguess: i32,
                depth: u8,
                mut num_nodes: &mut u64,
                end_time: time::Timespec)
                -> Option<i32> {
        let mut g = firstguess;
        let mut upper = i32::max_value();
        let mut lower = i32::min_value();
        let mut beta: i32;
        let mut direction = 1i32;
        let mut step_size = 1i32;
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
                if direction < 0i32 {
                    direction = 1i32;
                } else {
                    direction += 1i32;
                }
                upper = g;
                step_size = direction;
                if step_size > 10i32 {
                    step_size = 10i32;
                }
            } else {
                if direction > 0i32 {
                    direction = -1i32;
                } else {
                    direction -= 1i32;
                }

                lower = g;

                step_size = -direction;
                if step_size > 10i32 {
                    step_size = 10i32;
                }
            }

            if lower >= upper {
                break;
            }
        }

        Some(g)
    }

    pub fn choose_move(&mut self, s: &Box<State>) -> Direction {
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
