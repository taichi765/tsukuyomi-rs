use std::{collections::HashMap, time::Duration};

use crate::{
    engine::EngineCommand,
    fixture::Fixture,
    functions::{Function, FunctionType},
};

pub struct TimeLine {
    id: usize,
    name: String,
    tracks: Vec<Track>,
}

impl TimeLine {
    fn new(id: usize, name: &str) -> Self {
        Self {
            id,
            name: name.into(),
            tracks: Vec::new(),
        }
    }
}

impl Function for TimeLine {
    fn run(
        &mut self,
        _function_infos: &std::collections::HashMap<usize, super::FunctionInfo>,
        _fixtures: &HashMap<usize, Fixture>,
        _tick_duration: Duration,
    ) -> Vec<EngineCommand> {
        // TODO
        return vec![];
    }

    fn function_type(&self) -> FunctionType {
        FunctionType::TimeLine
    }
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
}

struct Track {
    index: usize,
    name: String,
    items: Vec<TrackItem>,
}

struct TrackItem {
    function_id: usize,
    start_time: Duration,
}
