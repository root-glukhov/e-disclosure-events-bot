mod db_context;
mod bot_handlers;
mod parser;
mod postman;

use dotenvy_macro::{self, dotenv};
use once_cell::sync::OnceCell;
use teloxide::{
    Bot, 
    dptree, 
    types::Update, 
    dispatching::UpdateFilterExt, 
    prelude::Dispatcher
};


static DB: OnceCell<db_context::Database> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Database
    let mut db = db_context::Database::open().await?;
    db.migrate().await?;
    DB.set(db).unwrap();

    // Postman
    tokio::spawn(async move {
        postman::start().await;
    });

    // Teloxide
    let bot = Bot::new(dotenv!("TELOXIDE_TOKEN"));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(bot_handlers::message_handler))
        .branch(Update::filter_callback_query().endpoint(bot_handlers::callback_handler)
    );

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
        
    Ok(())
}
