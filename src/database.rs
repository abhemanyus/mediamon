use sqlx::{migrate, sqlite::SqlitePoolOptions, SqlitePool};

pub struct Database {
    pool: SqlitePool
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Io(#[from] dotenv::Error),
    #[error(transparent)]
    Migrate(#[from] migrate::MigrateError),
}
pub type Result<T> = std::result::Result<T, Error>;

impl Database {
    pub async fn new() -> Result<Self> {
        let database_url = dotenv::var("DATABASE_URL")?;
        let pool = SqlitePoolOptions::new().connect(&database_url).await?;
        migrate!().run(&pool).await?;
        Ok (Self {pool})
    }
}
