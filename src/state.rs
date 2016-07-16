use std::hash;
use std::hash::{Hash, Hasher};

use fnv::FnvHasher;

use game::Game;
use hero::Hero;
use tile::Tile;
use direction::Direction;
use position::Position;

#[derive(Clone, Deserialize, Debug, Eq)]
pub struct State {
    pub game: Game,
    pub hero: Hero,
    pub token: String,
    #[serde(rename="viewUrl")]
    pub view_url: String,
    #[serde(rename="playUrl")]
    pub play_url: String,
}

pub struct UnmakeInfo {
    heroes: [Hero; 4],
    tiles: Vec<(Position, Tile)>,
}

impl hash::Hash for State {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.game.hash(state);
        self.token.hash(state);
    }
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        let mut sh = FnvHasher::default();
        self.hash(&mut sh);
        let shash = sh.finish();

        let mut oh = FnvHasher::default();
        other.hash(&mut oh);
        let ohash = oh.finish();

        shash == ohash
    }
}

impl State {
    fn kill(&mut self, hero_id: usize, killer_id: usize, umi: &mut UnmakeInfo) {
        // println!("{} killed by {}", hero_id, killer_id);

        if killer_id > 0 {
            self.game.heroes[killer_id - 1].mine_count += self.game.heroes[hero_id - 1].mine_count;
        }
        self.game.heroes[hero_id - 1].mine_count = 0;

        let mut mpos = vec![];
        for pos in &self.game.board.mine_pos {
            match self.game.board.tile_at(pos) {
                Tile::Mine(hid) if hid == hero_id => mpos.push(pos.clone()),
                _ => (),
            }
        }

        // println!("{:?}", mpos);

        for ref pos in mpos {
            umi.tiles.push((*pos, self.game.board.tile_at(pos)));
            self.game.board.put_tile(pos, Tile::Mine(killer_id));
        }

        for i in 1..4 {
            if self.game.heroes[((hero_id - 1) + i) % 4].pos ==
               self.game.heroes[hero_id - 1].spawn_pos {
                let killed_id = self.game.heroes[((hero_id - 1) + i) % 4].id;
                {
                    self.kill(killed_id, hero_id, umi)
                }
            }
        }

        let old_pos = &self.game.heroes[hero_id - 1].pos.clone();
        let new_pos = &self.game.heroes[hero_id - 1].spawn_pos.clone();

        umi.tiles.push((*old_pos, Tile::Hero(hero_id)));
        self.game.board.put_tile(old_pos, Tile::Air);
        umi.tiles.push((*new_pos, Tile::Air));
        self.game.board.put_tile(new_pos, Tile::Hero(hero_id));
        self.game.heroes[hero_id - 1].pos = *new_pos;
        self.game.heroes[hero_id - 1].life = 100;
    }

    pub fn get_moves(&self) -> Vec<Direction> {
        let mut result: Vec<Direction> = Vec::with_capacity(5);
        let h = &self.game.heroes[self.game.turn % 4];

        if self.game.turn > self.game.max_turns {
            return result;
        }

        if h.crashed {
            result.push(Direction::Stay);
            return result;
        }

        for dir in &[Direction::North, Direction::East, Direction::South, Direction::West] {
            let t =
                self.game.board.tile_at(&self.game.heroes[self.game.turn % 4].pos.neighbor(*dir));

            match t {
                Tile::Wall | Tile::Hero(_) => (),               
                Tile::Mine(x) if x == h.id => (),
                Tile::Air | Tile::Mine(_) => result.push(*dir),
                Tile::Tavern => {
                    if h.gold >= 2 {
                        result.push(*dir);
                    }
                }               
            }
        }

        result.push(Direction::Stay);
        result
    }

    pub fn make_move(&mut self, direction: Direction) -> UnmakeInfo {
        let mut result = UnmakeInfo {
            heroes: [self.game.heroes[0].clone(),
                     self.game.heroes[1].clone(),
                     self.game.heroes[2].clone(),
                     self.game.heroes[3].clone()],
            tiles: Vec::with_capacity(32),
        };

        let h_idx = (self.game.turn % 4) as usize;
        let mut hero_died = false;

        match self.game.board.tile_at(&self.game.heroes[h_idx].pos.neighbor(direction)) {
            Tile::Wall | Tile::Hero(_) => (),         
            Tile::Tavern => {
                if self.game.heroes[h_idx].gold >= 2 {
                    self.game.heroes[h_idx].gold -= 2;
                    self.game.heroes[h_idx].life += 50;
                    if self.game.heroes[h_idx].life > 100 {
                        self.game.heroes[h_idx].life = 100;
                    }
                }
            }
            Tile::Air => {
                result.tiles.push((self.game.heroes[h_idx].pos, Tile::Hero(h_idx + 1)));
                self.game.board.put_tile(&self.game.heroes[h_idx].pos, Tile::Air);
                result.tiles.push((self.game.heroes[h_idx].pos.neighbor(direction), Tile::Air));
                self.game.board.put_tile(&self.game.heroes[h_idx].pos.neighbor(direction),
                                         Tile::Hero(h_idx + 1));
                self.game.heroes[h_idx].pos = self.game.heroes[h_idx].pos.neighbor(direction);
            }
            Tile::Mine(hero_id) => {
                if hero_id != h_idx + 1 {
                    if self.game.heroes[h_idx].life <= 20 {
                        hero_died = true;
                        self.kill(h_idx + 1, 0, &mut result)
                    } else {
                        if hero_id > 0 {
                            self.game.heroes[hero_id - 1].mine_count -= 1;
                        }
                        self.game.heroes[h_idx].mine_count += 1;
                        self.game.heroes[h_idx].life -= 20;
                        result.tiles.push((self.game.heroes[h_idx].pos.neighbor(direction),
                                           self.game
                            .board
                            .tile_at(&self.game.heroes[h_idx].pos.neighbor(direction))));
                        self.game.board.put_tile(&self.game.heroes[h_idx].pos.neighbor(direction),
                                                 Tile::Mine(h_idx + 1));
                    }
                }
            }
        }

        if !hero_died {
            for i in 0..(4 as usize) {
                if i == h_idx {
                    continue;
                }

                if self.game.heroes[i].pos.manhattan_distance(&self.game.heroes[h_idx].pos) == 1 {
                    if self.game.heroes[i].life <= 20 {
                        self.kill(i + 1, h_idx + 1, &mut result);
                    } else {
                        self.game.heroes[i].life -= 20;
                    }
                }
            }
        }

        self.game.heroes[h_idx].gold += self.game.heroes[h_idx].mine_count as u16;

        if self.game.heroes[h_idx].life > 1 {
            self.game.heroes[h_idx].life -= 1;
        }

        let ldir: &'static str = direction.into();

        self.game.heroes[h_idx].last_dir = String::from(ldir);
        self.hero = self.game.heroes[self.hero.id - 1].clone();
        self.game.turn += 1;

        if self.game.turn == self.game.max_turns {
            self.game.finished = true;
        }

        result
    }

    pub fn unmake_move(&mut self, umi: UnmakeInfo) {
        self.game.finished = false;
        self.game.turn -= 1;
        self.game.heroes = umi.heroes;
        self.hero = self.game.heroes[self.hero.id - 1].clone();
        for &(pos, t) in umi.tiles.iter().rev() {
            self.game.board.put_tile(&pos, t)
        }
    }
}
