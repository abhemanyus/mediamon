use sqlx::{migrate, sqlite::SqlitePoolOptions, QueryBuilder, Sqlite, SqlitePool};

#[derive(Clone)]
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

    pub async fn get_tag_names(&self, tag_ids: &[i32]) -> Result<Vec<(i32, String)>> {
        let mut query_builder =
            QueryBuilder::<Sqlite>::new("SELECT tag_id, name FROM tag WHERE tag_id IN (");
        let mut separated = query_builder.separated(",");
        for tag_id in tag_ids {
            separated.push_bind(tag_id);
        }
        separated.push_unseparated(")");
        query_builder.push("ORDER BY tag_id");
        let query = query_builder
            .build_query_as::<(i32, String)>()
            .fetch_all(&self.pool)
            .await?;
        Ok(query)
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
