use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
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
    last_message: String,
    league: String,

    item2_name: Option<String>,

    cost_number: Option<String>,
    cost_currency: Option<String>,

    stash: Option<String>,
    left: Option<String>,
    top: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("can't parse line: {0}")]
    ParseError(String),
}

static TRADE_MSG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"@(?<type>(?:From|To)) (?<guild>(?:<.+>){0,1})\s*(?<char>\w+):"#).unwrap()
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
    let guild = matches.get(2).map(|e| {
        e.as_str()
            .trim_matches(|c| c == '<' || c == '>')
            .to_string()
    });
    let char = matches["char"].to_string();
    (t, guild, char)
}

static ENG_STASH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"\(stash tab "(?<stash>.*)"; position: left (?<left>\d+), top (?<top>\d+)\)"#)
        .unwrap()
});
static ENG_QUALITY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"level (?<lvl>\d+) (?<quality>\d+)% (?<item>.*)"#).unwrap());

static ENG_MSGS: Lazy<Vec<Regex>> = Lazy::new(|| {
    let a = Regex::new(r#"Hi, I would like to buy your (?<item>[\w\s,]+) listed for (?<cost>[\d\.]+) (?<currency>[\w-]+) in (?<league>\w+)"#).unwrap();
    let b =
        Regex::new(r#"Hi, I would like to buy your (?<item>[\w\s,]+) in (?<league>\w+)"#).unwrap();
    let c = Regex::new(
        r#"Hi, I'd like to buy your (?<item>[\w\s,]+) for my (?<item2>[\w\s]+) in (?<league>\w+)"#,
    )
    .unwrap();

    vec![a, b, c]
});

pub struct Model {
    trades: HashMap<String, TradeInfo>,
    outgoing_callback: Box<dyn Fn(&TradeInfo) + Send>,
    incoming_callback: Box<dyn Fn(&TradeInfo) + Send>,
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
        F: Fn(&TradeInfo) + Send + 'static,
    {
        self.outgoing_callback = Box::new(cb);
    }

    pub fn incoming_subscribe<F>(&mut self, cb: F)
    where
        F: Fn(&TradeInfo) + Send + 'static,
    {
        self.incoming_callback = Box::new(cb);
    }

    pub fn try_add(&mut self, line: &str) -> Result<(), ModelError> {
        if !is_trade(line) {
            return Ok(());
        }

        let (trade_type, _, char) = type_person_info(line);
        println!("char parsed: {}", char);

        let trade_info = if let Some(v) = self.trades.values_mut().find(|v| v.player_name == char) {
            println!("old trade info: {}", line);
            v
        } else {
            let mut matches = None;
            for re in ENG_MSGS.iter() {
                let c = re.captures(line);
                if c.is_some() {
                    matches = c;
                    break;
                }
            }
            if matches.is_none() {
                return Err(ModelError::ParseError(line.to_string()));
            }
            println!("parsed line: {}", line);
            let matches = matches.unwrap();
            let match_quality = ENG_QUALITY.captures(line);
            let id = Uuid::new_v4();
            let localtime = chrono::Local::now().time();
            let trade_info = TradeInfo {
                id: id.to_string(),
                typ: trade_type,
                cost_currency: matches.name("currency").map(|e| e.as_str().to_string()),
                item_name: matches["item"].to_string(),
                cost_number: matches.name("cost").map(|e| e.as_str().to_string()),
                last_message: String::new(),
                player_name: char,
                time: localtime.format("%H:%M").to_string(),
                league: matches["league"].to_string(),
                stash: matches.name("stash").map(|e| e.as_str().to_string()),
                left: matches.name("left").map(|e| e.as_str().to_string()),
                top: matches.name("top").map(|e| e.as_str().to_string()),
                // bugged
                item2_name: match_quality.map(|m| m["item"].to_string()),
            };
            self.trades.entry(id.to_string()).or_insert(trade_info)
        };
        trade_info.last_message = line.to_string();

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
    use crate::test_utilities::Callable;

    #[test]
    fn regexps() {
        let msg = r#"@From <TestGuild> 匚丹匚丹几丹: Hi, I would like to buy your Aegis Aurora Champion Kite Shield listed for 5 awakened-sextant in Ancestor (stash tab "~b/o 4.99 awakened-sextant"; position: left 9, top 7)"#;

        assert!(is_trade(msg));
        assert!(!is_trade("some specific information"));

        let (trade_type, guild, char) = type_person_info(msg);
        assert_eq!(trade_type, TradeType::Incoming);
        assert_eq!(guild, Some("TestGuild".to_string()));
        assert_eq!(char, "匚丹匚丹几丹".to_string());
    }

    #[test]
    fn messages() {
        let mut model = Model::new();
        let clb = Callable::new();
        {
            let clb2 = clb.clone();
            model.outgoing_subscribe(move |og| {
                clb2.call();
            });
            let clb = clb.clone();
            model.incoming_subscribe(move |og| {
                clb.call();
            });
        }

        let msgs = [
            r#"@From <TestGuild> 匚丹匚丹几丹: Hi, I would like to buy your Aegis Aurora Champion Kite Shield listed for 5 awakened-sextant in Ancestor (stash tab "~b/o 4.99 awakened-sextant"; position: left 9, top 7)"#,
            r#"@To 匚丹匚丹几丹: Hi, I would like to buy your Aegis Aurora Champion Kite Shield listed for 5 awakened-sextant in Ancestor (stash tab "~b/o 4.99 awakened-sextant"; position: left 9, top 7)"#,
            r#"16642: 2020/06/23 17:07:09 1067255656 b5c [INFO Client 10768] @From sethmera: Hi, I would like to buy your Onslaught Bind Chain Belt listed for 1 awakened-sextant in Harvest (stash tab "~price 1 chaos"; position: left 2, top 1)"#,
            r#"2023/10/13 01:54:50 1054470421 cffb0719 [INFO Client 30680] @From SambaLe: Hi, I would like to buy your The Pandemonius, Jade Amulet listed for 4 divine in Ancestor (stash tab "pub"; position: left 11, top 1)"#,
        ];

        for m in msgs.iter() {
            model.try_add(m).unwrap();
        }
        assert_eq!(clb.count(), msgs.len() as u64);
    }
}
