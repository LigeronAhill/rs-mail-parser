use super::{ParseResult, StockItem};
use calamine::{open_workbook_auto_from_rs, Data, DataType, Range, Reader};
use std::io::Cursor;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use tracing::error;

pub fn parser(files: Vec<Vec<u8>>) -> ParseResult {
    let supplier = "fancy".to_string();
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
    ParseResult { supplier, items }
}

fn parse(table: Range<Data>, tx: Sender<StockItem>) {
    let re = regex::Regex::new(r#"^([A-z]+)\s.+$"#).unwrap();
    let mut name = String::new();
    let mut second_name = String::new();
    for row in table.rows() {
        let temp_name = row.first().and_then(|d| d.get_string()).unwrap_or_default();
        if re.is_match(temp_name) {
            let temp_second_name = row.get(4).and_then(|d| d.get_string()).unwrap_or_default();
            second_name = temp_second_name.to_string();
            name = format!("{temp_name} {second_name}");
        } else if let Some(current) = row.get(4).and_then(|d| d.to_string().trim().parse::<f64>().ok()) {
            let reserved = row.get(6).and_then(|d| d.to_string().trim().parse::<f64>().ok()).unwrap_or_default();
            let stock = current - reserved;
            let item = StockItem { name: name.clone(), stock };
            if tx.send(item).is_err() {
                error!("Error sending item...")
            }
        }
    }
}

