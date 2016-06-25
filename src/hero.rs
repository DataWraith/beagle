use position::Position;

#[derive(Clone, Deserialize, Debug)]
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
