mod db_context;

use once_cell::sync::OnceCell;


static DB: OnceCell<db_context::Database> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = db_context::Database::open().await?;
    db.migrate().await?;
    DB.set(db).unwrap();

    Ok(())
}
