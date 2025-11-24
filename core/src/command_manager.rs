use std::sync::Arc;
use std::sync::RwLock;
use tsukuyomi_core::doc::Doc;
use tsukuyomi_core::doc::DocCommand;
use tsukuyomi_core::doc::DocEvent;
use tsukuyomi_core::doc::DocObserver;

pub struct CommandManager {
    commands: Vec<Box<dyn DocCommand>>,
    current_index: usize,
    observers: Vec<Arc<RwLock<dyn DocObserver>>>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            current_index: 0,
            observers: Vec::new(),
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

    pub fn subscribe(&mut self, observer: Arc<RwLock<dyn DocObserver>>) {
        self.observers.push(observer);
    }

    fn notify(&mut self, event: DocEvent) {
        for observer in &mut self.observers {
            let mut observer = observer.write().unwrap();
            observer.on_doc_event(event);
        }
    }
}
