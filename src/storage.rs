use crate::xlparser::ParseResult;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::{Datetime, Id, Thing};
use surrealdb::Surreal;
const NAMESPACE: &str = "safira";
const DBNAME: &str = "assortment";
pub struct Storage {
    db: Surreal<Client>,
}
impl Storage {
    async fn clear(&self, supplier: &str) -> Result<()> {
        let sql = "
            DELETE raw_stock WHERE supplier = $supplier RETURN BEFORE;
        ";
        let mut result = self.db.query(sql).bind(("supplier", supplier)).await?;
        let deleted: Vec<DbStockItem> = result.take(0)?;
        tracing::info!("Deleted {} records of supplier {supplier}", deleted.len());
        Ok(())
    }
    pub async fn update(&self, pr: ParseResult) -> Result<()> {
        let supplier = pr.supplier.clone();
        self.clear(&supplier).await?;
        let obj = DbStockItem::from_parse_result(pr);
        let created: Vec<DbStockItem> = self.db.insert("raw_stock").content(obj).await?;
        tracing::info!(
            "Created {} new records of supplier {supplier}",
            created.len()
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DbStockItem {
    id: Thing,
    supplier: String,
    name: String,
    stock: f64,
    updated: Datetime,
}
impl DbStockItem {
    fn new(supplier: String, name: String, stock: f64, updated: Datetime) -> Self {
        let id = Thing {
            tb: "raw_stock".to_string(),
            id: Id::rand(),
        };
        Self {
            id,
            supplier,
            name,
            stock,
            updated,
        }
    }
    fn from_parse_result(pr: ParseResult) -> Vec<DbStockItem> {
        use rayon::prelude::*;
        let supplier = pr.supplier;
        pr.items
            .par_iter()
            .map(|i| DbStockItem::new(supplier.clone(), i.name.clone(), i.stock, pr.date.clone()))
            .collect()
    }
}
pub async fn new(user: &str, pass: &str, addr: &str) -> Result<Storage> {
    let db = Surreal::new::<Ws>(addr).await?;

    db.signin(Root {
        username: user,
        password: pass,
    })
    .await?;

    db.use_ns(NAMESPACE).use_db(DBNAME).await?;
    Ok(Storage { db })
}
