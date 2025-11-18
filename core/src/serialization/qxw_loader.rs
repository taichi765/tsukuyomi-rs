use itertools::Itertools;
use serde::Deserialize;
use std::{collections::HashMap, fs};

use crate::{
    engine::Engine,
    functions::{Scene, SceneValue},
};

#[derive(Deserialize, Debug)]
struct XmlWorkSpace {
    #[serde(rename = "Engine")]
    engine: XmlEngine,
}

#[derive(Deserialize, Debug)]
struct XmlEngine {
    #[serde(rename = "$value", default)]
    childs: Vec<XmlEngineChild>,
}

#[derive(Deserialize, Debug)]
enum XmlEngineChild {
    Function(XmlFunction),
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
enum XmlFunction {
    Scene(XmlScene),
    Chaser(XmlChaser),
    Collection(XmlCollection),
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Debug)]
struct XmlSpeed {
    #[serde(rename = "@FadeIn")]
    fadein: usize,
    #[serde(rename = "@Duration")]
    hold: usize,
    #[serde(rename = "@FadeOut")]
    fadeout: usize,
}

#[derive(Deserialize, Debug)]
struct XmlScene {
    #[serde(rename = "@ID")]
    id: usize,
    #[serde(rename = "@Name")]
    name: String,
    #[serde(rename = "Speed")]
    speed: XmlSpeed,
    #[serde(rename = "$value")]
    values: Vec<XmlSceneValue>,
}

#[derive(Deserialize, Debug)]
struct XmlSceneValue {
    #[serde(rename = "ID")]
    fixture_id: usize,
    #[serde(rename = "$text")]
    values: String,
}

#[derive(Deserialize, Debug)]
struct XmlChaser {
    #[serde(rename = "@ID")]
    id: usize,
    #[serde(rename = "@Name")]
    name: String,
}

#[derive(Deserialize, Debug)]
struct XmlCollection {
    #[serde(rename = "ID")]
    id: usize,
    #[serde(rename = "Name")]
    name: String,
}

fn deserialize(path: &str) -> Result<XmlEngine, String> {
    let xml_content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let workspace_data: XmlWorkSpace =
        quick_xml::de::from_str(&xml_content).map_err(|e| e.to_string())?;
    Ok(workspace_data.engine)
}

fn parse_scene_value(data: &str) -> Result<SceneValue, String> {
    let numbers: Vec<u16> = data
        .split(',')
        .map(|s| s.parse::<u16>().map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    let mut value_map: HashMap<u16, u8> = HashMap::new();

    numbers.into_iter().tuples().for_each(|(channel, value)| {
        value_map.insert(channel, value as u8).unwrap();
    });
    Ok(value_map)
}

pub fn load(engine: &mut Engine, path: &str) -> Result<(), String> {
    let engine_data = deserialize(path)?;
    for child in engine_data.childs {
        if let XmlEngineChild::Function(function) = child {
            match function {
                XmlFunction::Scene(scene_data) => {
                    let mut scene = Scene::new(scene_data.id, &scene_data.name);
                    scene_data
                        .values
                        .iter()
                        .map(|v| (v.fixture_id, parse_scene_value(&v.values).unwrap()))
                        .collect::<HashMap<usize, SceneValue>>()
                        .into_iter()
                        .for_each(|(fixture_id, value)| scene.push_value(fixture_id, value));

                    engine.push_function(Box::new(scene)).unwrap();
                }
                XmlFunction::Chaser(chaser) => {
                    println!("{}", chaser.name)
                }
                XmlFunction::Collection(collection) => println!("{}", collection.name),
                XmlFunction::Other => (),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Engine;

    #[test]
    fn test_loader_works() {
        let mut engine = Engine::new();
        load(&mut engine, "hebirote.qxw").unwrap();
    }
}
