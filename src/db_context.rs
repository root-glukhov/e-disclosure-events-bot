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
        sqlx::query(MIGRATIONS).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn add_event(
        &self, 
        telegram_id: &str, 
        company_id: i32,
        company_name: &str
    ) -> Result<String, sqlx::Error> {
        // ct_id - company table id
        let ct_id = sqlx::query_scalar::<_, i32>(
            "
            INSERT OR IGNORE INTO companies (company_id, name) VALUES (?1, ?2);
            SELECT id FROM companies WHERE company_id=?1 LIMIT 1;
            "
        )
        .bind(company_id)
        .bind(company_name)
        .fetch_one(&self.pool)
        .await?;

        // st_id - subscriptions table id
        let st_id = sqlx::query_scalar::<_, i32>(
            "SELECT id FROM subscriptions WHERE telegram_id = ?1 AND company_id = ?2"
        )
        .bind(telegram_id)
        .bind(ct_id)
        .fetch_one(&self.pool)
        .await;

        if let Ok(_) = st_id {
            let reply = format!("Компания {} уже есть в отслеживаемых", company_name);
            return Ok(reply);
        }

        sqlx::query(
            "INSERT INTO subscriptions (telegram_id, company_id) VALUES (?1, ?2)"
        )
        .bind(telegram_id)
        .bind(ct_id)
        .execute(&self.pool)
        .await?;
        
        Ok(
            "Компания добавлена к списку отслеживаемых!".to_string()
        )
    }

    pub async fn get_events(
        &self,
        telegram_id: &str
    ) -> Result<Vec<(i32, String)>, sqlx::Error> {

        Ok(sqlx::query_as::<_, (i32, String)>(
            "
            SELECT S.id, C.name FROM subscriptions S
            INNER JOIN companies C ON S.company_id = C.id
            WHERE telegram_id = ?
            "
        )
        .bind(telegram_id)
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn delete_event(
        &self,
        id: i32
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM subscriptions WHERE id = ?"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_subscriptions(&self) -> Result<Vec<(i64, i32, String)>, sqlx::Error> {
        let res = sqlx::query_as::<_, (i64, i32, String)>(
            "
            SELECT S.telegram_id, C.company_id, C.name FROM companies C 
            INNER JOIN subscriptions S ON C.id = S.company_id
            "
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res)
    }
}