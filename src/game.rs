use std::hash;

use board::Board;
use hero::Hero;

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Game {
    pub id: String,
    pub turn: usize,
	#[serde(rename="maxTurns")]
    pub max_turns: usize,
    pub heroes: [Hero; 4],
    pub board: Board,
    pub finished: bool,
}

impl Clone for Game {
    fn clone(&self) -> Game {
        Game{
            id: self.id.clone(),
            turn: self.turn,
            max_turns: self.max_turns,
            heroes: [
              self.heroes[0].clone(), self.heroes[1].clone(), self.heroes[2].clone(), self.heroes[3].clone()
            ],
            board: self.board.clone(),
            finished: self.finished,
        }
    }
}

impl hash::Hash for Game {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.turn.hash(state);
        self.max_turns.hash(state);
        self.heroes[0].hash(state);
        self.heroes[1].hash(state);
        self.heroes[2].hash(state);
        self.heroes[3].hash(state);
        self.board.hash(state);
        self.finished.hash(state);
    }
}
