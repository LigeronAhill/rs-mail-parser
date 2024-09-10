use super::{ParseResult, StockItem};
use calamine::{open_workbook_auto_from_rs, Data, DataType, Range, Reader};
use std::io::Cursor;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use tracing::error;

pub fn parser(files: Vec<Vec<u8>>) -> ParseResult {
    let supplier = "opus".to_string();
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
                    error!("Error opening file from opus attachments: {e:?}");
                    continue;
                }
            }
        }
    });
    let items = rx.iter().collect();
    ParseResult { supplier, items }
}

fn parse(table: Range<Data>, tx: Sender<StockItem>) {
    let mut brand = String::new();
    let mut pt = String::new();
    for row in table.rows() {
        if let Some(stock) = row.get(5).and_then(|data| data.get_float()) {
            if let Some(raw_name) = row
                .first()
                .and_then(|data| data.get_string().map(|w| w.to_string()))
            {
                if PRODUCT_TYPES.contains(&raw_name.as_str()) {
                    pt = raw_name;
                    continue;
                } else if BRANDS.contains(&raw_name.as_str()) {
                    brand = raw_name;
                    continue;
                } else if stock > 5.0 {
                    let name = format!("{pt} {brand} {raw_name}");
                    let item = StockItem { name, stock };
                    if tx.send(item).is_err() {
                        error!("Error sending item...")
                    }
                }
            }
        }
    }
}

const PRODUCT_TYPES: [&str; 16] = [
    "Грязезащита",
    "Интернет-магазин",
    "Искусственная трава",
    "Ковровая плитка",
    "Контрактные обои",
    "Мебель",
    "Осветительное оборудование",
    "Паркет",
    "ПВХ плитка",
    "ПВХ рулонные",
    "Подвесные потолки",
    "Резиновые покрытия",
    "Рулонные ковровые покрытия",
    "Сопутствующие товары",
    "Стеновые панели",
    "Фальшполы",
];

const BRANDS: [&str; 56] = [
    "Betap",
    "Уличные покрытия",
    "Desoma Grass",
    "Betap",
    "Bloq",
    "Innovflor",
    "Interface",
    "IVC (Mohawk)",
    "Tapibel",
    "Виниловые покрытия",
    "Флизелиновые обои под покраску",
    "Ресторация",
    "CSVT",
    "Navigator",
    "ЛЕД-Эффект",
    "РУСВИТАЛЭЛЕКТРО",
    "Barlinek",
    "Coswick",
    "Royal Parket",
    "Карелия Упофлор",
    "паркет VOLVO",
    "Спортивные системы",
    "ADO Floor",
    "Interface",
    "KBS floor",
    "Tarkett",
    "Vertigo",
    "Гомогенный",
    "С защитой от статического электричества / токопроводящий",
    "Спортивный",
    "МЕТАЛЛИЧЕСКИЕ ПОТОЛКИ",
    "МЕТАЛЛИЧЕСКИЕ ПРОСТЫЕ ПОТОЛКИ",
    "МИНЕРАЛЬНЫЕ ПОТОЛКИ",
    "Beka Rubber",
    "Desoma Rubber Fitness Premium",
    "Beaulieu International Group",
    "Betap Tufting B.V.",
    "Condor carpets",
    "Haima",
    "Luxemburg",
    "Синтелон",
    "Материалы для монтажа и ухода",
    "Плинтус",
    "Подложка",
    "Шнур сварочный",
    "FORTIKA CDF",
    "FORTIKA HPL",
    "Swiss KRONO CDF",
    "CBI (Си-Би-Ай)",
    "Fortika",
    "Perfaten, АСП",
    "Конструктор (Аксиома)(Айрон)",
    "Панели других производителей",
    "Сопутствующие товары",
    "Стойки других производителей",
    "Стрингеры",
];
