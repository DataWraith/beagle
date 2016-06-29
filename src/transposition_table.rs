
#[derive(Default, Clone)]
pub struct Entry {
	pub mv: &'static str,
	pub hash: u64,
	pub lower: f32,
	pub upper: f32,
	pub turn: u16,
	pub age: u16,
}

pub struct Table {
	num_entries : u64,
	always: Vec<Entry>,
	depthpref: Vec<Entry>,
}

impl Table {
	pub fn new(num_entries : u64) -> Table {
		if num_entries % 2 == 1 {
			panic!("Num entries must be even, got {}.", num_entries);
		}
		
		Table{
			num_entries: num_entries / 2,
			always: vec![Entry::default(); (num_entries / 2) as usize],
			depthpref: vec![Entry::default(); (num_entries / 2) as usize],
		}
	}
	
	pub fn probe(&self, hash : u64) -> Option<Entry> {
		let idx = (hash % self.num_entries) as usize;
		
		if self.depthpref[idx].hash == hash {
			return Some(self.depthpref[idx].clone());
		}
			
		if self.always[idx].hash == hash {
			return Some(self.always[idx].clone());
		}
		
		
		return None
	}
	
	pub fn store(&mut self, e : Entry) {
		let idx = (e.hash % self.num_entries) as usize;
		
		if self.depthpref[idx].turn <= e.turn || self.depthpref[idx].age + 15 < e.age {
			self.depthpref[idx] = e;
			return;
		}
		
		self.always[idx] = e;
	}
}