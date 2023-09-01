use sea_orm::{error::DbErr, RuntimeErr};

pub fn get_sql_error(error: DbErr) -> sqlx::error::ErrorKind {
    match error {
        DbErr::Query(RuntimeErr::SqlxError(sql_error)) => match sql_error {
            sqlx::Error::Database(e) => e.kind(),
            _ => panic!("Unexpected database error: {:?}", sql_error),
        },
        _ => panic!("Unexpected database error: {:?}", error),
    }
}
