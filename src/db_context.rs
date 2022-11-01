use sqlx::SqlitePool;

const MIGRATIONS: &str = 
    "

    CREATE TABLE IF NOT EXISTS companies (
        id          INTEGER PRIMARY KEY,
        company_id  INTEGER UNIQUE NOT NULL,
        name        NVARCHAR(100) NOT NULL
    );

    
    CREATE TABLE IF NOT EXISTS subscriptions (
        id          INTEGER PRIMARY KEY,
        telegram_id INTEGER NOT NULL,
        company_id  INTEGER NOT NULL,
        FOREIGN KEY (company_id) REFERENCES companies(id)
    );

    ";

#[derive(Debug)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn open() -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect("./sql.db3?mode=rwc").await?;
        Ok(Database { pool })
    }

    pub async fn migrate(&mut self) -> Result<(), sqlx::Error> {
        sqlx::query(MIGRATIONS).execute(&self.pool)
        .await?;

        Ok(())
    }
}