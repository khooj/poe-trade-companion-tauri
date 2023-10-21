use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum TradeType {
    Incoming,
    Outgoing,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TradeInfo {
    id: String,
    #[serde(rename = "type")]
    typ: TradeType,
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

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("can't parse line")]
    ParseError,
}

static TRADE_MSG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"@(?<type>(From|To)) (?<guild>(\<.+\>){0,1})\s*(?<char>.+):"#).unwrap()
});

fn is_trade(line: &str) -> bool {
    TRADE_MSG.is_match(line)
}

fn type_person_info(line: &str) -> (TradeType, Option<String>, String) {
    let matches = TRADE_MSG.captures(line).unwrap();
    let t = match &matches["type"] {
        "From" => TradeType::Incoming,
        "To" => TradeType::Outgoing,
        _ => panic!("unknown trade type"),
    };
    let guild = matches.get(2).map(|e| e.as_str().to_string());
    let char = matches["char"].to_string();
    (t, guild, char)
}

static ENG_INCOMING: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"Hi, I would like to buy your (?<item>[\w\s]+) listed for (?<cost>[\d\.]+) (?<currency>[\w-]+) in (?<league>\w+) \(stash tab "(?<stash>.*)"; position: left (?<left>\d+), top (?<top>\d+)\)"#).unwrap()
});

pub struct Model {
    trades: HashMap<Uuid, TradeInfo>,
    outgoing_callback: Box<dyn Fn(TradeInfo) + Send>,
    incoming_callback: Box<dyn Fn(TradeInfo) + Send>,
}

impl Model {
    pub fn new() -> Model {
        Model {
            trades: HashMap::new(),
            outgoing_callback: Box::new(|_| {}),
            incoming_callback: Box::new(|_| {}),
        }
    }

    pub fn outgoing_subscribe<F>(&mut self, cb: F)
    where
        F: Fn(TradeInfo) + Send + 'static,
    {
        self.outgoing_callback = Box::new(cb);
    }

    pub fn incoming_subscribe<F>(&mut self, cb: F)
    where
        F: Fn(TradeInfo) + Send + 'static,
    {
        self.incoming_callback = Box::new(cb);
    }

    pub fn try_add(&mut self, line: &str) -> Result<(), ModelError> {
        if !is_trade(line) {
            return Ok(());
        }

        let (trade_type, _, char) = type_person_info(line);
        let matches = ENG_INCOMING.captures(line).ok_or(ModelError::ParseError)?;
        let id = Uuid::new_v4();
        let localtime = chrono::Local::now().time();
        let trade_info = TradeInfo {
            id: id.to_string(),
            typ: trade_type,
            cost_currency: matches["currency"].to_string(),
            item_name: matches["item"].to_string(),
            cost_number: matches["cost"].to_string(),
            last_message: line.to_string(),
            player_name: char,
            time: localtime.format("%H:%M").to_string(),
            league: matches["league"].to_string(),
            stash: matches["stash"].to_string(),
            left: matches["left"].to_string(),
            top: matches["top"].to_string(),
        };
        self.trades.insert(id, trade_info.clone());
        match trade_info.typ {
            TradeType::Incoming => (self.incoming_callback)(trade_info),
            TradeType::Outgoing => (self.outgoing_callback)(trade_info),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages() -> Result<(), Box<dyn std::error::Error>> {
        let mut model = Model::new();
        model.outgoing_subscribe(|og| {
            assert_eq!(og.cost_currency, "awakened-sextant");
        });
        model.incoming_subscribe(|og| {
            assert_eq!(og.cost_currency, "awakened-sextant");
        });

        let msgs = [
            r#"@From <TestGuild> 匚丹匚丹几丹: Hi, I would like to buy your Aegis Aurora Champion Kite Shield listed for 5 awakened-sextant in Ancestor (stash tab "~b/o 4.99 awakened-sextant"; position: left 9, top 7)"#,
            r#"@To 匚丹匚丹几丹: Hi, I would like to buy your Aegis Aurora Champion Kite Shield listed for 5 awakened-sextant in Ancestor (stash tab "~b/o 4.99 awakened-sextant"; position: left 9, top 7)"#,
            r#"16642: 2020/06/23 17:07:09 1067255656 b5c [INFO Client 10768] @From sethmera: Hi, I would like to buy your Onslaught Bind Chain Belt listed for 1 awakened-sextant in Harvest (stash tab "~price 1 chaos"; position: left 2, top 1)"#,
        ];

        for m in msgs {
            model.try_add(m)?;
        }
        Ok(())
    }
}
