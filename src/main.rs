mod db_context;
mod bot_handlers;
mod parser;

use dotenv_codegen::dotenv;
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
    let mut db = db_context::Database::open().await?;
    db.migrate().await?;
    DB.set(db).unwrap();

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

    // let _db = DB.get().unwrap();
    // let res = _db.add_event("123456", 1234, "company_name").await?;

    // println!("{}", res);

    // let res = _db.get_events("123456").await?;

    // for row in res {
    //     let r = _db.delete_event(row.id).await?;
    //     println!("{}", r);
    // }

    Ok(())
}
