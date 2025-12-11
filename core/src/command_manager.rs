use crate::{commands::DocCommand, doc::DocHandle};

pub struct CommandManager {
    commands: Vec<Box<dyn DocCommand>>,
    current_index: usize,
    doc_handle: DocHandle,
}

// TODO: DocCommand->Eventの生成
// FIXME: イベント通知とundo/redoで責務が二つある
impl CommandManager {
    pub fn new(doc_handle: DocHandle) -> Self {
        Self {
            commands: Vec::new(),
            current_index: 0,
            doc_handle,
        }
    }

    pub fn execute(&mut self, mut command: Box<dyn DocCommand>) -> Result<(), String> {
        command.apply(&self.doc_handle)?;
        self.commands.truncate(self.current_index);
        self.commands.push(command);
        self.current_index = self.commands.len();
        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), String> {
        if self.current_index == 0 {
            return Err("no command to undo".into());
        }
        self.commands[self.current_index - 1].revert(&self.doc_handle)?;
        self.current_index -= 1;
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), String> {
        if self.current_index == self.commands.len() {
            return Err("no command to redo".into());
        }
        self.commands[self.current_index + 1].apply(&self.doc_handle)?;
        self.current_index += 1;
        Ok(())
    }
}
