use crate::{
    doc::{Doc, DocCommand},
    functions::FunctionData,
};

pub struct AddFunction {
    function: Option<FunctionData>,
    function_id: usize,
}

impl AddFunction {
    pub fn new(function: FunctionData) -> Self {
        Self {
            function_id: function.id(),
            function: Some(function),
        }
    }
}

impl DocCommand for AddFunction {
    fn apply(&mut self, doc: &mut Doc) -> Result<(), String> {
        if let Some(f) = self.function.take() {
            doc.add_function(f);
            Ok(())
        } else {
            Err("function is already moved".into())
        }
    }
    fn revert(&mut self, doc: &mut Doc) -> Result<(), String> {
        if let Some(f) = doc.remove_function(self.function_id) {
            self.function = Some(f);
            Ok(())
        } else {
            Err("function is already removed from doc".into())
        }
    }
}
