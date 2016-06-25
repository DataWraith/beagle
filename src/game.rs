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
