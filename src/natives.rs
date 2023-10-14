use std::collections::HashMap;

use crate::vm::{self, State, Value};

pub type Function = Box<dyn Fn(&mut State) -> vm::Res<Value>>;

pub struct Natives {
    map: HashMap<&'static [u8], u32>,
    array: Vec<(u8, Function)>,
}

impl Natives {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            array: Vec::new(),
        }
    }

    pub fn function<F>(&mut self, name: &'static [u8], argc: u8, f: F)
    where
        F: Fn(&mut State) -> vm::Res<Value> + 'static,
    {
        let index = self.array.len();
        if index > 0xFFFFFFFF {
            panic!("Natives count overflow.");
        }
        self.array.push((argc, Box::new(f)));
        if self.map.insert(name, index as u32).is_some() {
            panic!("Native \"{name:?}\" already exists.");
        }
    }

    pub fn get_index(&self, name: &[u8]) -> Option<u32> {
        self.map.get(name).cloned()
    }

    pub fn get_by_index(&self, index: u32) -> &(u8, Function) {
        &self.array[index as usize]
    }
}
