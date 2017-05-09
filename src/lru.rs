pub struct LRU<T> {
    cur: u32,
    keys: [T; 20],
    lrus: [u32; 20],
}

impl <T:Copy + Eq> LRU<T> {
    pub fn new(default_entry: T) -> LRU<T> {
        LRU {
            cur: 1,
            keys: [default_entry; 20],
            lrus: [0; 20],
        }
    }

    pub fn query(&mut self, entry: T) -> u32 {
        for i in 0..20 {
            if self.keys[i] == entry {
                let result = self.cur - self.lrus[i];
                self.lrus[i] = self.cur;
                self.cur += 1;
                return result
            }
        }

        255
    }

    pub fn insert(&mut self, entry: T) {
        let mut min_lru = 0xFFFFFFFF;
        let mut min_idx = 0xFFFFFFFF;

        for i in 0..20 {
            if self.keys[i] == entry {
                self.lrus[i] = self.cur;
                self.cur += 1;
                return
            }

            if self.lrus[i] < min_lru {
                min_idx = i;
                min_lru = self.lrus[i];

                if min_lru == 0 {
                    break
                }
            }
        }

        self.keys[min_idx] = entry;
        self.lrus[min_idx] = self.cur;
        self.cur += 1;
    }
}

