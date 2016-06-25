use board::Board;
use hero::Hero;

#[derive(Deserialize, Debug)]
pub struct Game {
    pub id: String,
    pub turn: u16,
	#[serde(rename="maxTurns")]
    pub max_turns: u16,
    pub heroes: [Hero; 4],
    pub board: Board,
    pub finished: bool,
}
