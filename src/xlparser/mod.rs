pub mod opus;
pub mod fancy;

#[derive(Debug)]
pub struct StockItem {
    pub name: String,
    pub stock: f64,
}

#[derive(Debug)]
pub struct ParseResult {
    pub supplier: String,
    pub items: Vec<StockItem>,
}