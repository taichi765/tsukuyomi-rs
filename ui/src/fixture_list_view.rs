use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

use slint::{ComponentHandle, Model, ModelRc, ToSharedString, VecModel, Weak};
use tsukuyomi_core::{
    doc::{DocEvent, DocEventBus, DocObserver, DocStore},
    readonly::ReadOnly,
};

use crate::{AppWindow, FixtureListStore, FixtureModel, ManufacturerModel};

pub fn setup_fixture_list_view(
    ui: &AppWindow,
    doc: ReadOnly<DocStore>,
    event_bus: &mut DocEventBus,
) -> Arc<RwLock<FixtureListViewController>> {
    let controller = Arc::new(RwLock::new(FixtureListViewController::new(
        ui.as_weak(),
        doc,
    )));
    event_bus.subscribe(Arc::downgrade(&controller) as _);

    // Reset dummy model
    ui.global::<FixtureListStore>()
        .set_model(Rc::new(VecModel::from(Vec::new())).into());
    controller
}

pub struct FixtureListViewController {
    ui_handle: Weak<AppWindow>,
    doc: ReadOnly<DocStore>,
}

impl FixtureListViewController {
    fn new(ui_handle: Weak<AppWindow>, doc: ReadOnly<DocStore>) -> Self {
        Self { ui_handle, doc }
    }
}

impl DocObserver for FixtureListViewController {
    fn on_doc_event(&mut self, event: &DocEvent) {
        match event {
            DocEvent::FixtureDefInserted(id) => {
                let ui = self.ui_handle.unwrap();
                let store = ui.global::<FixtureListStore>();

                let doc = self.doc.read();
                let def = doc.get_fixture_def(id).unwrap();
                let new_manufacturer = def.manufacturer();
                let new_fixture_model = FixtureModel {
                    id: id.to_shared_string(),
                    name: def.model().to_shared_string(),
                };
                let mut manufactures: Vec<ManufacturerModel> = store.get_model().iter().collect();
                let mut is_updated = false;
                for m in &mut manufactures {
                    if m.manufacturer != new_manufacturer {
                        continue;
                    }
                    let filtered: Vec<(usize, FixtureModel)> = m
                        .fixtures
                        .iter()
                        .enumerate()
                        .filter(|(_, fxt)| fxt.id == id.to_shared_string())
                        .collect();
                    if filtered.len() != 0 {
                        m.fixtures
                            .set_row_data(filtered[0].0, new_fixture_model.clone());
                        is_updated = true;
                    } else {
                        m.fixtures
                            .set_row_data(m.fixtures.iter().len(), new_fixture_model.clone());
                        is_updated = true;
                    }
                    break;
                }
                if !is_updated {
                    manufactures.push(ManufacturerModel {
                        expanded: false,
                        fixtures: Rc::new(VecModel::from(vec![new_fixture_model])).into(),
                        manufacturer: new_manufacturer.to_shared_string(),
                    });
                }
                store.set_model(Rc::new(VecModel::from(manufactures)).into());
            }
            DocEvent::FixtureDefRemoved(id) => {}
            _ => (),
        }
    }
}
