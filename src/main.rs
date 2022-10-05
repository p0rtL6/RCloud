use once_cell::sync::OnceCell;
use sea_orm::{Database, DatabaseConnection};

mod dirs_table;
mod files_table;
mod handlers;

pub use dirs_table::prelude::*;
pub use files_table::prelude::*;
pub use handlers::config::Config;
pub use handlers::database::*;
pub use handlers::filesystem::*;
pub use handlers::server::*;

pub static DATABASE_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::new();
pub static CONFIG: OnceCell<Config> = OnceCell::new();

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_toml("./config.toml");

    CONFIG.set(config).unwrap();

    let db = Database::connect(&CONFIG.get().unwrap().database.path)
        .await
        .unwrap();

    DATABASE_CONNECTION.set(db).unwrap();

    create_schema().await;
    create_root_dir().await;

    let (shutdown_sender, shutdown_receiver) = tokio::sync::broadcast::channel(1);

    tokio::spawn(async move {
        start_server().await.unwrap();
    });

    tokio::select! {
        biased;
        _ = tokio::signal::ctrl_c() => { // This broke again :(
            shutdown_sender.send(()).unwrap();
        }
        _ = watch_fs(shutdown_receiver) => {},
    }

    Ok(())
}
