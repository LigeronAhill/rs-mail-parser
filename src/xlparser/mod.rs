pub mod opus;

#[derive(Debug)]
pub struct StockItem {
    name: String,
    stock: f64,
}

#[derive(Debug)]
pub struct ParseResult {
    supplier: String,
    items: Vec<StockItem>,
}