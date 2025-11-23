use crate::doc::{Doc, DocCommand};

pub mod doc;
pub mod static_scene;
pub mod timeline;

pub struct CommandManager {
    commands: Vec<Box<dyn DocCommand>>,
    current_index: usize,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            current_index: 0,
        }
    }

    pub fn execute(
        &mut self,
        mut command: Box<dyn DocCommand>,
        doc: &mut Doc,
    ) -> Result<(), String> {
        command.apply(doc)?;
        self.commands.truncate(self.current_index);
        self.commands.push(command);
        self.current_index = self.commands.len();
        Ok(())
    }

    pub fn undo(&mut self, doc: &mut Doc) -> Result<(), String> {
        if self.current_index == 0 {
            return Err("no command to undo".into());
        }
        self.commands[self.current_index - 1].revert(doc)?;
        self.current_index -= 1;
        Ok(())
    }

    pub fn redo(&mut self, doc: &mut Doc) -> Result<(), String> {
        if self.current_index == self.commands.len() {
            return Err("no command to redo".into());
        }
        self.commands[self.current_index + 1].apply(doc)?;
        self.current_index += 1;
        Ok(())
    }
}
