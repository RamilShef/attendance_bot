mod config;
mod db;
mod bot;
mod utils;

use anyhow::Result;
use teloxide::{prelude::*, dptree};
use dotenvy::dotenv;
use tracing_subscriber;

use sea_orm::{Database, DatabaseConnection, ConnectionTrait, sea_query::*};

use db::entities::{students, attendance};
use bot::commands::Command;
use bot::handlers::{handle_command, handle_text, handle_callback};

async fn run_migrations(db: &DatabaseConnection) -> Result<()> {
    let backend = db.get_database_backend();

    // Создание таблицы students
    let create_students = Table::create()
        .table(students::Entity)
        .if_not_exists()
        .col(ColumnDef::new(students::Column::Id).integer().not_null().auto_increment().primary_key())
        .col(ColumnDef::new(students::Column::TgId).big_integer().not_null().unique_key())
        .col(ColumnDef::new(students::Column::Name).string().not_null())
        .to_owned();

    db.execute(backend.build(&create_students)).await?;

    // Создание таблицы attendance
    let create_attendance = Table::create()
        .table(attendance::Entity)
        .if_not_exists()
        .col(ColumnDef::new(attendance::Column::StudentId).integer().not_null())
        .col(ColumnDef::new(attendance::Column::Date).string().not_null())
        .col(ColumnDef::new(attendance::Column::Present).boolean().not_null().default(false))
        .primary_key(
            Index::create()
                .col(attendance::Column::StudentId)
                .col(attendance::Column::Date)
        )
        .foreign_key(
            ForeignKey::create()
                .from(attendance::Entity, attendance::Column::StudentId)
                .to(students::Entity, students::Column::Id)
                .on_delete(ForeignKeyAction::Cascade)
        )
        .to_owned();

    db.execute(backend.build(&create_attendance)).await?;

    // Создание индекса на поле date
    let create_index = Index::create()
        .name("idx_attendance_date")
        .table(attendance::Entity)
        .col(attendance::Column::Date)
        .to_owned();

    if let Err(e) = db.execute(backend.build(&create_index)).await {
        if !e.to_string().contains("already exists") {
            return Err(e.into());
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env();
    let db = Database::connect(&config.database_url).await?;

    if let Err(e) = run_migrations(&db).await {
        eprintln!("❌ Ошибка миграции: {}", e);
        return Err(e);
    }

    let bot = Bot::new(&config.bot_token);

    let handler = dptree::entry()
        .branch(Update::filter_message().filter_command::<Command>().endpoint(handle_command))
        .branch(Update::filter_message().endpoint(handle_text))
        .branch(Update::filter_callback_query().endpoint(handle_callback));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![db, config])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}