use crate::engine::EngineCommand;
use crate::fixture::Fixture;
use std::any::Any;
use std::collections::HashMap;
use std::time::Duration;

mod chaser;
mod collection;
mod fader;
mod scene;

pub use chaser::Chaser;
pub use collection::Collection;
pub(crate) use fader::Fader;
pub use scene::Scene;
pub use scene::SceneValue;

pub trait Function: Any {
    //コマンドパターン
    //実際にUniverseやプラグインに書き込むのは責務外
    fn run(
        &mut self, //可変借用はselfのみ
        function_infos: &HashMap<usize, FunctionInfo>,
        fixtures: &HashMap<usize, Fixture>,
        context: &Context,
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
}

#[derive(Clone, Copy)]
pub struct FunctionInfo {
    pub id: usize,
    pub function_type: FunctionType,
}

pub struct Context {
    pub tick_duration: Duration,
}
