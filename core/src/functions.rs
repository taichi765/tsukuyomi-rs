use crate::engine::FunctionCommand;
use crate::fixture::Fixture;
use std::any::Any;
use std::collections::HashMap;
use std::time::Duration;

mod chaser;
mod collection;
mod fader;
mod static_scene;
mod timeline;

pub use chaser::Chaser;
pub use collection::Collection;
pub(crate) use fader::Fader;
pub use static_scene::SceneValue;
pub use static_scene::StaticScene;

pub trait Function: Any {
    //コマンドパターン
    //実際にUniverseやプラグインに書き込むのは責務外
    fn run(
        &mut self, //可変借用はselfのみ
        function_infos: &HashMap<usize, FunctionInfo>,
        fixtures: &HashMap<usize, Fixture>,
        tick_duration: Duration,
    ) -> Vec<EngineCommand>;
    fn function_type(&self) -> FunctionType;
    fn id(&self) -> usize;
    fn name(&self) -> &str;
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
