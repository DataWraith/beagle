use rand;
use rand::{Rng};

pub struct ZobristTable {
	pub keys: [u64; 12*35*35],
}

impl Default for ZobristTable {
	fn default() -> ZobristTable {
		let mut result = ZobristTable{
			keys: [0; 12*35*35],
		};
		
		for i in 0..(12*35*35) {
			result.keys[i] = rand::thread_rng().gen();
		}
		
		result
	}
}

pub static mut ZOBRIST : ZobristTable = ZobristTable{keys: [0; 12*35*35]};