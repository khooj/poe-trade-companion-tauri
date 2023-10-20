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
    league: String,
    stash: String,
    left: String,
    top: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomingTrade {
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

static ENG_INCOMING: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?<time>(\[.*\]\s){0,1})@(?<char>.+) Hi, I would like to buy your (?<item>[\w\s]+) listed for (?<cost>[\d\.]+) (?<currency>[\w-]+) in (?<league>\w+) \(stash tab "(?<stash>.*)"; position: left (?<left>\d+), top (?<top>\d+)\)"#).unwrap()
});

pub struct Model {
    outgoing: HashMap<Uuid, OutgoingTrade>,
    outgoing_callback: Box<dyn Fn(OutgoingTrade) + Send>,
    incoming: HashMap<Uuid, IncomingTrade>,
    incoming_callback: Box<dyn Fn(IncomingTrade) + Send>,
}

impl Model {
    pub fn new() -> Model {
        Model {
            outgoing: HashMap::new(),
            outgoing_callback: Box::new(|_| {}),
            incoming: HashMap::new(),
            incoming_callback: Box::new(|_| {}),
        }
    }

    pub fn outgoing_subscribe<F>(&mut self, cb: F)
    where
        F: Fn(OutgoingTrade) + Send + 'static,
    {
        self.outgoing_callback = Box::new(cb);
    }

    pub fn incoming_subscribe<F>(&mut self, cb: F)
    where
        F: Fn(IncomingTrade) + Send + 'static,
    {
        self.incoming_callback = Box::new(cb);
    }

    pub fn try_add(&mut self, line: &str) -> Result<(), ModelError> {
        let matches = ENG_INCOMING.captures(line).ok_or(ModelError::ParseError)?;
        let id = Uuid::new_v4();
        let outgoing = OutgoingTrade {
            id: id.to_string(),
            cost_currency: matches["currency"].to_string(),
            item_name: matches["item"].to_string(),
            cost_number: matches["cost"].to_string(),
            last_message: line.to_string(),
            player_name: matches["char"].to_string(),
            time: matches["time"].to_string(),
            league: matches["league"].to_string(),
            stash: matches["stash"].to_string(),
            left: matches["left"].to_string(),
            top: matches["top"].to_string(),
        };
        self.outgoing.insert(id, outgoing.clone());
        (self.outgoing_callback)(outgoing);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outgoing_messages() -> Result<(), Box<dyn std::error::Error>> {
        let mut model = Model::new();
        model.outgoing_subscribe(|og| {
            assert_eq!(og.cost_currency, "awakened-sextant");
        });

        let msgs = [
            r#"[19:55] @匚丹匚丹几丹 Hi, I would like to buy your Aegis Aurora Champion Kite Shield listed for 5 awakened-sextant in Ancestor (stash tab "~b/o 4.99 awakened-sextant"; position: left 9, top 7)"#,
            r#"@匚丹匚丹几丹 Hi, I would like to buy your Aegis Aurora Champion Kite Shield listed for 5 awakened-sextant in Ancestor (stash tab "~b/o 4.99 awakened-sextant"; position: left 9, top 7)"#,
        ];

        for m in msgs {
            model.try_add(m)?;
        }
        Ok(())
    }
}
