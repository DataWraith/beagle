use mv::Move;
 
#[derive(Default, Clone)]
pub struct Entry {
	pub mv: Move,
	pub hash: u64,
	pub lower: i32,
	pub upper: i32,
	pub turn: u16,
	pub age: u16,
}

pub struct Table {
	num_entries : u64,
	always: Vec<Box<Entry>>,
	depthpref: Vec<Box<Entry>>,
}

impl Table {
	pub fn new(num_entries : u64) -> Table {
		if num_entries % 2 == 1 {
			panic!("Num entries must be even, got {}.", num_entries);
		}
		
		Table{
			num_entries: num_entries / 2,
			always: vec![Box::new(Entry::default()); (num_entries / 2) as usize],
			depthpref: vec![Box::new(Entry::default()); (num_entries / 2) as usize],
		}
	}
	
	pub fn probe(&self, hash : u64) -> Option<Entry> {
		let idx = (hash % self.num_entries) as usize;
		
		if self.depthpref[idx].hash == hash {
			let box ret = self.depthpref[idx].clone();
			return Some(ret);
		}
			
		if self.always[idx].hash == hash {
			let box ret = self.always[idx].clone();
			return Some(ret);
		}
		
		
		None
	}
	
	pub fn store(&mut self, e : Entry) {
		let idx = (e.hash % self.num_entries) as usize;
		
		if self.depthpref[idx].turn <= e.turn || self.depthpref[idx].age + 15 < e.age {
			self.depthpref[idx].mv = e.mv;
			self.depthpref[idx].hash = e.hash;
			self.depthpref[idx].lower = e.lower;
			self.depthpref[idx].upper = e.upper;
			self.depthpref[idx].turn = e.turn;
			self.depthpref[idx].age = e.age;
			
			return;
		}
		
		self.always[idx].mv = e.mv;
		self.always[idx].hash = e.hash;
		self.always[idx].lower = e.lower;
		self.always[idx].upper = e.upper;
		self.always[idx].turn = e.turn;
		self.always[idx].age = e.age;
	}
}