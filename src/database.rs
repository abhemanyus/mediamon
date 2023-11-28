use std::str::FromStr;

use sqlx::{
    migrate,
    sqlite::{SqliteConnectOptions, SqliteTypeInfo, SqliteValueRef},
    Decode, Sqlite, SqlitePool, Type,
};

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
        // let database_url = dotenv::var("DATABASE_URL")?;
        let sqlite_options = SqliteConnectOptions::from_str(":memory:")?
            .extension("vector0")
            .extension("vss0");
        let pool = SqlitePool::connect_with(sqlite_options).await?;
        // migrate!().run(&pool).await?;
        Ok(Self { pool })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct MyVec(Vec<f32>);

impl<'value> Decode<'value, Sqlite> for MyVec {
    fn decode(
        value: SqliteValueRef<'value>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let value = <&[u8] as Decode<Sqlite>>::decode(value)?;
        let value: Vec<f32> = value
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        Ok(Self(value))
    }
}

impl Type<Sqlite> for MyVec {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as Type<Sqlite>>::type_info()
    }
}

#[derive(sqlx::FromRow, Debug)]
struct MyRow {
    rowid: i64,
    distance: f32,
    a: MyVec,
}

#[tokio::test]
async fn connect_to_db() {
    let db = Database::new().await.unwrap();
    let sample_vec = Vec::from_iter((0..(9176 * 4)).map(|_| 1u8));
    sqlx::query(
        r"
    CREATE VIRTUAL TABLE vss_demo USING vss0(a(9176));
    INSERT INTO vss_demo(rowid, a)
        VALUES
            (1, ?);
    ",
    )
    .bind(&sample_vec)
    .execute(&db.pool)
    .await
    .unwrap();
    let row = sqlx::query_as::<_, MyRow>(
        r"
            SELECT
                rowid,
                distance,
                a
            FROM vss_demo
            WHERE VSS_SEARCH(a, ?)
            LIMIT 3
        ",
    )
    .bind(&sample_vec)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    println!("{:?}", row);
    panic!("WOOOOO");
}
