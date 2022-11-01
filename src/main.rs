mod db_context;

use once_cell::sync::OnceCell;


static DB: OnceCell<db_context::Database> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = db_context::Database::open().await?;
    db.migrate().await?;
    DB.set(db).unwrap();

    let _db = DB.get().unwrap();
    let res = _db.add_event("123456", 1234, "company_name").await?;

    println!("{}", res);
    Ok(())
}
