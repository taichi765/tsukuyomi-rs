pub mod doc_commands;
pub mod static_scene_commands;
pub mod timeline_commands;

use crate::doc::DocHandle;

pub trait DocCommand {
    fn apply(&mut self, doc: &DocHandle) -> Result<(), String>;

    fn revert(&mut self, doc: &DocHandle) -> Result<(), String>;
}
