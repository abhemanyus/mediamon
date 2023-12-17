use sqlx::{migrate, sqlite::SqlitePoolOptions, Execute, QueryBuilder, Sqlite, SqlitePool};

pub struct Database {
    pool: SqlitePool,
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
        Ok(Self { pool })
    }
}

#[test]
fn where_in() {
    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT * FROM population WHERE year IN (");
    let years = vec![2019, 2020, 2021];
    let mut separated = query_builder.separated(",");
    for year in years {
        separated.push_bind(year);
    }
    separated.push_unseparated(") ");
    let mut query = query_builder.build();
    let arguments = query.take_arguments().unwrap();
    dbg!(&arguments);
    let sql = query.sql();
    assert_eq!(sql, "SELECT * FROM population WHERE year IN (?,?,?)");
}
