use actix_web::{App, HttpServer};
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_toml("./config.toml");

    CONFIG.set(config).unwrap();

    let db = Database::connect(&CONFIG.get().unwrap().database.path)
        .await
        .unwrap();

    DATABASE_CONNECTION.set(db).unwrap();

    create_schema().await;
    create_root_dir().await;

    watch_fs().await;

    start_server().await

    // HttpServer::new(App::new)
        // .bind(("127.0.0.1", 8888))?
        // .run()
        // .await
}
