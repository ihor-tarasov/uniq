
pub struct Cycles {
    end_addresses: Vec<u32>,
    end_addresses_sizes: Vec<u32>,
    starts: Vec<u32>,
}

impl Cycles {
    pub fn new() -> Self {
        Self {
            end_addresses: Vec::new(),
            end_addresses_sizes: Vec::new(),
            starts: Vec::new(),
        }
    }

    pub fn push_end(&mut self, address: u32) -> bool {
        if let Some(address_size) = self.end_addresses_sizes.last_mut() {
            *address_size += 1;
            self.end_addresses.push(address);
            true
        } else {
            false
        }
    }

    pub fn start(&self) -> Option<u32> {
        self.starts.last().cloned()
    }

    pub fn push_start(&mut self, address: u32) {
        self.starts.push(address);
        self.end_addresses_sizes.push(0);
    }

    pub fn pop_start(&mut self) -> u32 {
        self.starts.pop().unwrap();
        self.end_addresses_sizes.pop().unwrap()
    }

    pub fn pop_end(&mut self) -> u32 {
        self.end_addresses.pop().unwrap()
    }
}
