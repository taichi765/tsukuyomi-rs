use crate::engine::FunctionCommand;
use crate::functions::chaser::ChaserRuntime;
use crate::functions::static_scene::StaticSceneRuntime;
use std::ops::Deref;
use std::time::Duration;

mod chaser;
mod collection;
mod fader;
mod static_scene;
mod timeline;

pub use chaser::ChaserData;
pub use collection::Collection;
pub(crate) use fader::Fader;
pub use static_scene::SceneValue;
pub use static_scene::StaticSceneData;

pub trait FunctionDataGetters {
    fn id(&self) -> usize;
    fn name(&self) -> &str;
}

pub enum FunctionData {
    StaticScene(StaticSceneData),
    Chaser(ChaserData),
}

impl Deref for FunctionData {
    type Target = dyn FunctionDataGetters;
    fn deref(&self) -> &Self::Target {
        match self {
            FunctionData::StaticScene(data) => data,
            FunctionData::Chaser(data) => data,
        }
    }
}

impl FunctionData {
    pub(crate) fn create_runtime(&self) -> Box<dyn FunctionRuntime> {
        match self {
            FunctionData::StaticScene(_) => Box::new(StaticSceneRuntime::new()),
            FunctionData::Chaser(_) => Box::new(ChaserRuntime::new()),
        }
    }
}

pub trait FunctionRuntime {
    fn run(&mut self, data: &FunctionData, tick_duration: Duration) -> Vec<FunctionCommand>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionType {
    Scene,
    Chaser,
    Fader,
    Collection,
    TimeLine,
    Other,
}

#[derive(Clone, Copy)]
pub struct FunctionInfo {
    pub id: usize,
    pub function_type: FunctionType,
}
