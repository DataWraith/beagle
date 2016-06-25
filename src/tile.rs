#[derive(Clone, Copy, Deserialize, Debug)]
pub enum Tile {
    Wall,
    Air,
    Tavern,
    Mine(usize),
    Hero(usize),
}

impl Default for Tile {
    fn default() -> Tile {
        Tile::Wall
    }
}
