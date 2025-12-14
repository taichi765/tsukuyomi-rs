use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use slint::{ComponentHandle, Model, ToSharedString, VecModel, Weak};
use tracing::debug;
use tsukuyomi_core::{
    command_manager::CommandManager,
    commands::doc_commands,
    doc::{DocEvent, DocEventBus, DocObserver, DocStore},
    fixture::Fixture,
    fixture_def::FixtureDefId,
    readonly::ReadOnly,
    universe::{DmxAddress, UniverseId},
};
use uuid::Uuid;

use crate::{AppWindow, FixtureListStore, FixtureModel, ManufacturerModel};

pub fn setup_fixture_list_view(
    ui: &AppWindow,
    doc: ReadOnly<DocStore>,
    event_bus: &mut DocEventBus,
    command_manager: Rc<RefCell<CommandManager>>,
) -> Arc<RwLock<FixtureListViewController>> {
    let controller = Arc::new(RwLock::new(FixtureListViewController::new(
        ui.as_weak(),
        ReadOnly::clone(&doc),
    )));
    event_bus.subscribe(Arc::downgrade(&controller) as _);

    let doc_clone = ReadOnly::clone(&doc);
    ui.global::<FixtureListStore>()
        .on_patch(move |universe, address, fixture_def_id, mode| {
            let universe_id = parse_universe_id(universe.as_str());
            let fixture_def_id =
                FixtureDefId::from(Uuid::parse_str(fixture_def_id.as_str()).unwrap());
            let fixture_name = {
                let doc = doc_clone.read();
                let fixture_def = doc.get_fixture_def(&fixture_def_id).unwrap(); // FIXME: unwrap
                let num = 0; // TODO: 同じFixtureDefを使うFixtureの数を取得する(DocStoreに追加？)
                format!("{}({})", fixture_def.model(), num)
            };
            let fixture = Fixture::new(
                fixture_name,
                universe_id,
                DmxAddress::new(address as usize).unwrap(),
                fixture_def_id,
                mode.to_string(),
            );
            command_manager
                .borrow_mut()
                .execute(Box::new(doc_commands::AddFixture::new(fixture)))
                .expect("failed to insert fixture"); // FIXME: エラーを返せるか？
        });

    let ui_handle = ui.as_weak();
    ui.global::<FixtureListStore>().on_get_modes(move |def_id| {
        let ui = ui_handle.unwrap();
        let store = ui.global::<FixtureListStore>();
        let fixture_model = store
            .get_model()
            .iter()
            .find_map(|m| m.fixtures.iter().find(|fxt| fxt.id == def_id))
            .unwrap();
        fixture_model.modes
    });

    let doc_clone = ReadOnly::clone(&doc);
    ui.global::<FixtureListStore>()
        .on_get_next_address(move |universe| {
            let universe_id = parse_universe_id(universe.as_str());
            doc_clone
                .read()
                .current_max_address(universe_id)
                .map_or(0, |adr| adr.value() as i32 + 1)
        });

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
            DocEvent::FixtureDefInserted(def_id) => {
                let ui = self.ui_handle.unwrap();
                let store = ui.global::<FixtureListStore>();
                let doc = self.doc.read();

                let def = doc.get_fixture_def(def_id).unwrap();
                let def_id = def_id.to_shared_string();

                let new_manufacturer = def.manufacturer();
                let new_fixture_model = FixtureModel {
                    id: def_id.to_shared_string(),
                    name: def.model().to_shared_string(),
                    modes: Rc::new(VecModel::from(
                        def.modes()
                            .keys()
                            .map(|s| s.to_shared_string())
                            .collect::<Vec<_>>(),
                    ))
                    .into(),
                };

                let mut manufacturers: Vec<ManufacturerModel> = store.get_model().iter().collect();
                if let Some(m) = manufacturers
                    .iter()
                    .find(|m| m.manufacturer == new_manufacturer)
                {
                    let existing_idx = m.fixtures.iter().position(|fxt| fxt.id == def_id);
                    let idx = existing_idx.unwrap_or_else(|| m.fixtures.iter().count());
                    m.fixtures.set_row_data(idx, new_fixture_model);
                } else {
                    manufacturers.push(ManufacturerModel {
                        expanded: false,
                        fixtures: Rc::new(VecModel::from(vec![new_fixture_model])).into(),
                        manufacturer: new_manufacturer.to_shared_string(),
                    });
                }

                store.set_model(Rc::new(VecModel::from(manufacturers)).into());
            }
            DocEvent::FixtureDefRemoved(id) => todo!(),
            _ => (),
        }
    }
}

impl Drop for FixtureListViewController {
    fn drop(&mut self) {
        debug!("FixtureListViewController is dropping");
    }
}

fn parse_universe_id(universe_name: &str) -> UniverseId {
    // TODO: nameを自由に付けられるようにする
    let universe_id = universe_name
        .split(" ")
        .collect::<Vec<&str>>()
        .get(1)
        .expect(
            "cusstom universe name is not supproted at the moment: expected `universe <number>`",
        )
        .parse::<u8>()
        .expect(
            "custom universe name is not supproted at the moment: expected `universe <number>`",
        ); // TODO: エラーを返せるか？
    UniverseId::new(universe_id)
}
