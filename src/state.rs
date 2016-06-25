use game::Game;
use hero::Hero;
use tile::Tile;

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct State {
    pub game: Game,
    pub hero: Hero,
    pub token: String,
	#[serde(rename="viewUrl")]
    pub view_url: String,
	#[serde(rename="playUrl")]
    pub play_url: String,
}

impl State {
    fn kill(&mut self, hero_id : usize, killer_id : usize) {
        println!("{} killed by {}", hero_id, killer_id);

        if killer_id > 0 {
            self.game.heroes[killer_id - 1].mine_count += self.game.heroes[hero_id - 1].mine_count;
        }
        self.game.heroes[hero_id - 1].mine_count = 0;

        let mut mpos = vec![];
        for pos in self.game.board.mine_pos.iter() {
            match self.game.board.tile_at(pos) {
                Tile::Mine(hid) if hid == hero_id => mpos.push(pos.clone()),
                _ => (),
            }
        }

        println!("{:?}", mpos);

        for ref pos in mpos {
            self.game.board.put_tile(pos, Tile::Mine(killer_id));
        }

        for i in 1..4 {
            if self.game.heroes[((hero_id - 1) + i) % 4].pos == self.game.heroes[hero_id - 1].spawn_pos {
                let killed_id = self.game.heroes[((hero_id - 1) + i) % 4].id;
                {
                    self.kill(killed_id, hero_id)
                }
            }
        }

        let ref old_pos = self.game.heroes[hero_id - 1].pos.clone();
        let ref new_pos = self.game.heroes[hero_id - 1].spawn_pos.clone();

        self.game.board.put_tile(old_pos, Tile::Air);
        self.game.board.put_tile(new_pos, Tile::Hero(hero_id));
        self.game.heroes[hero_id - 1].pos = new_pos.clone();
        self.game.heroes[hero_id - 1].life = 100;
    }

    pub fn make_move(&mut self, direction : &str) {
            let h_idx = (self.game.turn % 4) as usize;
        println!("Turn {}: {}, ({})", self.game.turn, direction, h_idx + 1);

            match self.game.board.tile_at(&self.game.heroes[h_idx].pos.neighbor(direction)) {
                Tile::Wall => (),
                Tile::Hero(_) => (),
                Tile::Tavern => if self.game.heroes[h_idx].gold >= 2 {
                    self.game.heroes[h_idx].gold -= 2;
                    self.game.heroes[h_idx].life += 50;
                    if self.game.heroes[h_idx].life > 100 {
                        self.game.heroes[h_idx].life = 100;
                    }
                },
                Tile::Air  => {
                    self.game.board.put_tile(&self.game.heroes[h_idx].pos, Tile::Air);
                    self.game.board.put_tile(&self.game.heroes[h_idx].pos.neighbor(direction), Tile::Hero(h_idx + 1));
                    self.game.heroes[h_idx].pos = self.game.heroes[h_idx].pos.neighbor(direction);
                },
                Tile::Mine(hero_id) => if hero_id != h_idx + 1 {
                    if self.game.heroes[h_idx].life <= 20 {
                        self.kill(h_idx + 1, 0)
                    } else {
                        if hero_id > 0 {
                            self.game.heroes[hero_id - 1].mine_count -= 1;
                        }
                        self.game.heroes[h_idx].mine_count += 1;
                        self.game.heroes[h_idx].life -= 20;
                        self.game.board.put_tile(&self.game.heroes[h_idx].pos.neighbor(direction), Tile::Mine(h_idx + 1));
                    }
                }
            }

        for i in 0..(4 as usize) {
            if self.game.heroes[i].pos.manhattan_distance(&self.game.heroes[h_idx].pos.clone()) == 1 {
                if self.game.heroes[i].life <= 20 {
                    self.kill(i + 1, h_idx  + 1);
                } else {
                    self.game.heroes[i].life -= 20;
                }
            }
        }

        self.game.heroes[h_idx].gold += self.game.heroes[h_idx].mine_count as u16;

        if self.game.heroes[h_idx].life > 1 {
            self.game.heroes[h_idx].life -= 1;
        }

        self.game.heroes[h_idx].last_dir = String::from(direction);

        if h_idx == self.hero.id - 1 {
            self.hero = self.game.heroes[h_idx].clone();
        }

        self.game.turn += 1;

        if self.game.turn == self.game.max_turns {
            self.game.finished = true;
        }
    }
}
