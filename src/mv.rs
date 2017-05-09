use direction::Direction;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Move {
    pub player: u8,
    pub directions: [Direction; 4],
}

impl Default for Move {
    fn default() -> Move {
        Move {
            player: 0,
            directions: [Direction::Stay,
                         Direction::Stay,
                         Direction::Stay,
                         Direction::Stay],
        }
    }
}
