use std::{collections::HashMap, sync::mpsc, thread};

use surrealdb::sql::Datetime;
use tracing::error;

mod carpetland;
mod fancy;
mod fenix;
mod fox;
mod opus;
mod ortgraph;
mod vvk;
mod zefir;

#[derive(Debug)]
pub struct StockItem {
    pub name: String,
    pub stock: f64,
}

#[derive(Debug)]
pub struct ParseResult {
    pub supplier: String,
    pub items: Vec<StockItem>,
    pub date: Datetime,
}
const SUPPLIERS: [&str; 8] = [
    "opus",
    "fox",
    "fancy",
    "carpetland",
    "zefir",
    "fenix",
    "vvk",
    "ortgraph",
];

pub fn parse(stock_map: HashMap<String, (Vec<Vec<u8>>, Datetime)>) -> Vec<ParseResult> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for supplier in SUPPLIERS {
            if let Some((files, received)) = stock_map.get(supplier) {
                match supplier {
                    "opus" => {
                        let tx = tx.clone();
                        let res = opus::parser(files.clone(), received.clone());
                        if tx.send(res).is_err() {
                            error!("Error sending result...");
                        }
                    }
                    "fox" => {
                        let tx = tx.clone();
                        let res = fox::parser(files.clone(), received.clone());
                        if tx.send(res).is_err() {
                            error!("Error sending result...");
                        }
                    }
                    "fancy" => {
                        let tx = tx.clone();
                        let res = fancy::parser(files.clone(), received.clone());
                        if tx.send(res).is_err() {
                            error!("Error sending result...");
                        }
                    }
                    "carpetland" => {
                        let tx = tx.clone();
                        let res = carpetland::parser(files.clone(), received.clone());
                        if tx.send(res).is_err() {
                            error!("Error sending result...");
                        }
                    }
                    "zefir" => {
                        let tx = tx.clone();
                        let res = zefir::parser(files.clone(), received.clone());
                        if tx.send(res).is_err() {
                            error!("Error sending result...");
                        }
                    }
                    "fenix" => {
                        let tx = tx.clone();
                        let res = fenix::parser(files.clone(), received.clone());
                        if tx.send(res).is_err() {
                            error!("Error sending result...");
                        }
                    }
                    "vvk" => {
                        let tx = tx.clone();
                        let res = vvk::parser(files.clone(), received.clone());
                        if tx.send(res).is_err() {
                            error!("Error sending result...");
                        }
                    }
                    "ortgraph" => {
                        let tx = tx.clone();
                        let res = ortgraph::parser(files.clone(), received.clone());
                        if tx.send(res).is_err() {
                            error!("Error sending result...");
                        }
                    }
                    _ => continue,
                }
            }
        }
    });
    rx.iter().collect()
}
