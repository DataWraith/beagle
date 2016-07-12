use std::hash;

use position::Position;

#[derive(Clone, Deserialize, Debug, Eq)]
pub struct Hero {
    pub id: usize,
    pub name: String,
	#[serde(default, rename="userId")]
    pub user_id: String,
    #[serde(default)]
    pub elo: u16,
    pub pos: Position,
    #[serde(default, rename="lastDir")]
    pub last_dir: String,
    pub life: u8,
    pub gold: u16,
	#[serde(rename="mineCount")]
    pub mine_count: u8,
	#[serde(rename="spawnPos")]
    pub spawn_pos: Position,
    pub crashed: bool,
}

impl PartialEq for Hero {
	fn eq(&self, other: &Hero) -> bool {
		self.pos == other.pos &&
		self.life == other.life &&
		self.gold == other.gold &&
		self.mine_count == other.mine_count &&
		self.id == other.id &&
		self.user_id == other.user_id &&
		self.crashed == other.crashed
	}
}

impl hash::Hash for Hero {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state);
		self.user_id.hash(state);
		self.pos.hash(state);
		self.life.hash(state);
		self.gold.hash(state);
		self.mine_count.hash(state);
		self.crashed.hash(state);
    }
}