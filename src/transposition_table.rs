use mv::Move;

#[repr(packed)]
#[derive(Default, Clone)]
pub struct Entry {
    pub mv: Move,
    pub hash: u64,
    pub lower: i32,
    pub upper: i32,
    pub depth: u16,
    pub age: u16,
}

pub struct Table {
    num_entries: u64,
    always: Vec<Entry>,
    depthpref: Vec<Entry>,
}

impl Table {
    pub fn new(num_entries: u64) -> Table {
        if num_entries % 2 == 1 {
            panic!("Num entries must be even, got {}.", num_entries);
        }

        Table {
            num_entries: num_entries / 2,
            always: vec![Entry::default(); (num_entries / 2) as usize],
            depthpref: vec![Entry::default(); (num_entries / 2) as usize],
        }
    }

    pub fn probe(&self, hash: u64) -> Option<Entry> {
        let idx = (hash % self.num_entries) as usize;

        if self.depthpref[idx].hash == hash {
            let ret = self.depthpref[idx].clone();
            return Some(ret);
        }

        if self.always[idx].hash == hash {
            let ret = self.always[idx].clone();
            return Some(ret);
        }

        None
    }

    pub fn store(&mut self, e: Entry) {
        let idx = (e.hash % self.num_entries) as usize;

        if self.depthpref[idx].depth <= e.depth || self.depthpref[idx].age + 15 < e.age {
            self.depthpref[idx].mv = e.mv;
            self.depthpref[idx].hash = e.hash;
            self.depthpref[idx].lower = e.lower;
            self.depthpref[idx].upper = e.upper;
            self.depthpref[idx].depth = e.depth;
            self.depthpref[idx].age = e.age;

            return;
        }

        self.always[idx].mv = e.mv;
        self.always[idx].hash = e.hash;
        self.always[idx].lower = e.lower;
        self.always[idx].upper = e.upper;
        self.always[idx].depth = e.depth;
        self.always[idx].age = e.age;
    }
}
