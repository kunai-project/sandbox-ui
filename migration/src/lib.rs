pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_samples_table;
mod m20250325_145201_create_analysis_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_samples_table::Migration),
            Box::new(m20250325_145201_create_analysis_table::Migration),
        ]
    }
}
