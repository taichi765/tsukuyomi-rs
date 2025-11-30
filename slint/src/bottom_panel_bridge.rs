use tsukuyomi_core::{
    doc::{Doc, DocEvent, DocObserver},
    readonly::ReadOnly,
};

pub struct BottomPanelBridge {
    doc: ReadOnly<Doc>,
}

impl BottomPanelBridge {
    pub fn new(doc: ReadOnly<Doc>) -> Self {
        Self { doc }
    }
}

impl DocObserver for BottomPanelBridge {
    fn on_doc_event(&mut self, event: &DocEvent) {
        match event {
            _ => (),
        }
    }
}
