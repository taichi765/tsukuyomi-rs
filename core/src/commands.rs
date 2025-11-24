pub trait DocCommand {
    fn apply(&mut self, doc: &mut Doc) -> Result<(), String>;

    fn revert(&mut self, doc: &mut Doc) -> Result<(), String>;
}

pub mod doc_commands;
pub mod static_scene_commands;
pub mod timeline_commands;
