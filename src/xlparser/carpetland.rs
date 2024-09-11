use super::{ParseResult, StockItem};
use calamine::{open_workbook_auto_from_rs, Data, Range, Reader};
use std::io::Cursor;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use surrealdb::sql::Datetime;
use tracing::error;

pub fn parser(files: Vec<Vec<u8>>, received: Datetime) -> ParseResult {
    let supplier = "carpetland".to_string();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for file in files {
            let cursor = Cursor::new(file);
            match open_workbook_auto_from_rs(cursor) {
                Ok(mut wb) => {
                    let sheets = wb.sheet_names();
                    for sheet in sheets {
                        if let Ok(table) = wb.worksheet_range(&sheet) {
                            let tx = tx.clone();
                            thread::spawn(move || parse(table, tx));
                        }
                    }
                }
                Err(e) => {
                    error!("Error opening file from fancy attachments: {e:?}");
                    continue;
                }
            }
        }
    });
    let items = rx.iter().collect();
    ParseResult {
        supplier,
        items,
        date: received,
    }
}

fn parse(table: Range<Data>, tx: Sender<StockItem>) {
    for row in table.rows() {
        if let Some(stock) = row
            .get(5)
            .and_then(|d| d.to_string().trim().parse::<f64>().ok())
        {
            let brand = row.first().map(|d| d.to_string()).unwrap_or_default();
            let collection = row.get(1).map(|d| d.to_string()).unwrap_or_default();
            let color = row.get(2).map(|d| d.to_string()).unwrap_or_default();
            let width = row.get(3).map(|d| d.to_string()).unwrap_or_default();
            let name = format!("{brand} {collection} {color} {width}");
            let item = StockItem {
                name: name.clone(),
                stock,
            };
            if tx.send(item).is_err() {
                error!("Error sending item...")
            }
        }
    }
}
