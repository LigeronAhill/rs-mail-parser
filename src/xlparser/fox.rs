use super::{ParseResult, StockItem};
use calamine::{open_workbook_auto_from_rs, Data, DataType, Range, Reader};
use std::io::Cursor;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use surrealdb::sql::Datetime;
use tracing::error;

pub fn parser(files: Vec<Vec<u8>>, received: Datetime) -> ParseResult {
    let supplier = "fox".to_string();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for file in files {
            let cursor = Cursor::new(file);
            match open_workbook_auto_from_rs(cursor) {
                Ok(mut wb) => {
                    if let Some(sheet) = wb.sheet_names().first() {
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
    let mut name = String::new();
    let re = regex::Regex::new(r#"^[А-я]+\s.+$"#).unwrap();
    for row in table.rows() {
        let temp_name = row.get(2).and_then(|d| d.get_string()).unwrap_or_default();
        if re.is_match(temp_name) {
            name = temp_name.to_string();
        } else if let Some(stock) = row
            .get(6)
            .and_then(|d| d.to_string().trim().parse::<f64>().ok())
        {
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
