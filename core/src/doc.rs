use std::collections::HashMap;

use crate::{fixture::Fixture, functions::Function, universe::Universe};

pub struct Doc {
    universes: Vec<Universe>,
    fixtures: HashMap<usize, Fixture>,
    scenes: HashMap<usize, Box<dyn Function>>,
}
