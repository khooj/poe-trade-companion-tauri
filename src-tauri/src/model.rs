use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::Manager;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutgoingTrade {
    id: String,
    item_name: String,
    player_name: String,
    time: String,
    cost_number: String,
    cost_currency: String,
    last_message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct IncomingTrade {
    id: String,
    buyer: String,
    item: String,
    price: String,
    stash: String,
    last_message: String,
    time: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("can't parse line")]
    ParseError,
}

static ENG_INCOMING: Lazy<Regex> = Lazy::new(|| Regex::new(r"Hi (?<middle>.*) test").unwrap());

pub struct Model {
    outgoing: HashMap<Uuid, OutgoingTrade>,
    outgoing_queue: Vec<OutgoingTrade>,
    incoming: HashMap<Uuid, IncomingTrade>,
}

impl Model {
    pub fn new() -> Model {
        Model {
            outgoing: HashMap::new(),
            outgoing_queue: Vec::with_capacity(100),
            incoming: HashMap::new(),
        }
    }

    pub fn try_add(&mut self, line: &str) -> Result<(), ModelError> {
        let matches = ENG_INCOMING.captures(line).ok_or(ModelError::ParseError)?;
        let id = Uuid::new_v4();
        let outgoing = OutgoingTrade {
            id: id.to_string(),
            cost_currency: "divine".to_string(),
            item_name: "Shitty shit".to_string(),
            cost_number: "200".to_string(),
            last_message: "Hello".to_string(),
            player_name: matches["middle"].to_string(),
            time: "19:55".to_string(),
        };
        self.outgoing.insert(id, outgoing.clone());
        self.outgoing_queue.push(outgoing);
        Ok(())
    }

    pub fn get_new_outgoing(&mut self) -> Vec<OutgoingTrade> {
        self.outgoing_queue.drain(..).collect()
    }
}
