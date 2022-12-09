use sqlx::SqlitePool;

const MIGRATIONS: &str = 
    "

    CREATE TABLE IF NOT EXISTS companies (
        id              INTEGER PRIMARY KEY,
        company_id      INTEGER UNIQUE NOT NULL,
        company_name    NVARCHAR(100) NOT NULL
    );

    
    CREATE TABLE IF NOT EXISTS subs (
        id              INTEGER PRIMARY KEY,
        telegram_id     INTEGER NOT NULL,
        company         INTEGER NOT NULL,
        FOREIGN KEY (company) REFERENCES companies(id)
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

    pub async fn add_event(&self, telegram_id: &str, company_id: i32, company_name: &str) -> Result<String, String> {
        // ct_id - company table id
        let insert_company = sqlx::query_scalar::<_, i32>(
            "
            INSERT OR IGNORE INTO companies (company_id, company_name) VALUES (?1, ?2);
            SELECT id FROM companies WHERE company_id=?1 LIMIT 1;
            "
        )
        .bind(company_id)
        .bind(company_name)
        .fetch_one(&self.pool)
        .await;

        let ct_id = match insert_company {
            Ok(r) => r,
            Err(e) => { 
                println!("[Error]: {}", e);
                return Err("Ошибка добавления новой компании.".to_string());
            }
        };

        // sub_id - subscriptions table id
        let select_sub = sqlx::query_scalar::<_, i32>(
            "SELECT id FROM subs WHERE telegram_id = ?1 AND company = ?2"
        )
        .bind(telegram_id)
        .bind(ct_id)
        .fetch_one(&self.pool)
        .await;

        if let Ok(_) = select_sub {
            return Err(format!("Компания {} уже есть в отслеживаемых", company_name));
        }

        // Insert new subscription
        let insert_sub = sqlx::query(
            "INSERT INTO subs (telegram_id, company) VALUES (?1, ?2)"
        )
        .bind(telegram_id)
        .bind(ct_id)
        .execute(&self.pool)
        .await;

        if let Err(e) = insert_sub {
            println!("[Error]: {}", e);
            return Err("Ошибка создания подписки".to_string());
        } else {
            return Ok("Компания добавлена к списку отслеживаемых!".to_string());
        }
    }

    pub async fn get_events(
        &self,
        telegram_id: &str
    ) -> Result<Vec<(i32, String)>, sqlx::Error> {

        Ok(sqlx::query_as::<_, (i32, String)>(
            "
            SELECT S.id, C.company_name FROM subs S
            INNER JOIN companies C ON S.company = C.id
            WHERE telegram_id = ?
            "
        )
        .bind(telegram_id)
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn delete_event(&self, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query( "DELETE FROM subs WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_subs(&self) -> Result<Vec<(i64, i32, String)>, sqlx::Error> {
        let res = sqlx::query_as::<_, (i64, i32, String)>(
            "
            SELECT S.telegram_id, C.company_id, C.company_name FROM companies C 
            INNER JOIN subs S ON C.id = S.company
            "
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(res)
    }
}