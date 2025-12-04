use std::{rc::Rc, sync::mpsc::Sender};

use crate::{AppWindow, FaderLogic, FixtureEntityData, Preview2DLogic};
use slint::{Brush, Color, ComponentHandle, SharedString, VecModel};
use tsukuyomi_core::{engine::EngineCommand, fixture::FixtureId};
use uuid::Uuid;

pub fn adapt_fader_view(ui: &AppWindow, command_tx: Sender<EngineCommand>) {
    ui.global::<FaderLogic>().on_value_changed(
        move |_fixture_id: SharedString, channel: SharedString, value: i32| {
            println!("value changed!");
            command_tx
                .send(EngineCommand::SetLiveValue {
                    fixture_id: FixtureId::from(Uuid::nil()),
                    channel: channel.to_string(),
                    value: value as u8,
                })
                .expect("failed to send command to engine");
        },
    );
}

pub fn adapt_2d_preview(ui: &AppWindow) {
    let fixture_list: Vec<FixtureEntityData> = vec![
        FixtureEntityData {
            x: 10.0,
            y: 10.0,
            color: Brush::SolidColor(Color::from_rgb_u8(255, 255, 0)),
        },
        FixtureEntityData {
            x: 50.0,
            y: 50.0,
            color: Brush::SolidColor(Color::from_rgb_u8(0, 255, 127)),
        },
    ];
    ui.global::<Preview2DLogic>()
        .set_fixture_list(Rc::new(VecModel::from(fixture_list)).into());
}
