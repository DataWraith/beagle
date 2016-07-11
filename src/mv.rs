use direction::Direction;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Move {
	pub directions: [Direction; 4],
}

impl Default for Move {
	fn default() -> Move {
		Move{
			directions: [Direction::Stay, Direction::Stay, Direction::Stay, Direction::Stay]
		}
	}
}