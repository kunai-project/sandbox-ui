use std::{fs, path::Path};

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr};

#[inline(always)]
fn sqlite_url<P: AsRef<Path>>(p: P) -> String {
    format!("sqlite://{}", p.as_ref().to_string_lossy())
}

pub async fn set_up_db<P: AsRef<Path>>(p: P) -> Result<DatabaseConnection, DbErr> {
    // we create the DB file if not existing
    fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&p)
        .map_err(|e| DbErr::Custom(format!("failed to create sqlite database file: {e}")))?;

    let db = Database::connect(sqlite_url(&p)).await?;

    let db = match db.get_database_backend() {
        DbBackend::MySql => {
            unimplemented!()
        }
        DbBackend::Postgres => {
            unimplemented!()
        }
        DbBackend::Sqlite => db,
    };

    Migrator::up(&db, None).await?;

    Ok(db)
}
