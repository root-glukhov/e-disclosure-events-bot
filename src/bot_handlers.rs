use crate::{DB, parser};

use teloxide::{
    utils::command::BotCommands, 
    Bot, types::{Message, Me, InlineKeyboardMarkup, InlineKeyboardButton, CallbackQuery}, 
    requests::Requester, payloads::SendMessageSetters
};


#[derive(BotCommands)]
#[command(rename_rule = "lowercase", description = "Команды:")]
enum Command {
    #[command(description = "Помощь")]
    Help,
    #[command(description = "/add <название компании> - чтобы добавить к отслеживаемым")]
    Add(String),
    #[command(description = "/delete - чтобы удалить компанию из отслеживаемых")]
    Delete,
}

pub async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                bot.send_message(
                    msg.chat.id, 
                    Command::descriptions().to_string()
                )
                .await?;
            }
            Ok(Command::Add(query)) => {
                let companies = parser::search_company(&query).await.unwrap(); 
                let keyboard = make_keyboard(companies);
                
                bot.send_message(msg.chat.id, "Выберите компанию из найденных:")
                    .reply_markup(keyboard)
                    .await?;
            }

            Ok(Command::Delete) => {

            }

            Err(_) => {
                //bot.send_message(msg.chat.id, "Command not found!").await?;
            }
        }
    }

    Ok(())
}

pub async fn callback_handler(
    bot: Bot, 
    q: CallbackQuery
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(data) = q.data {
        bot.answer_callback_query(&q.id).await?;
        let message = q.message.unwrap();
        bot.delete_message(message.chat.id, message.id).await?;

        let a: Vec<&str> = data.split("|").collect();

        let _db = DB.get().unwrap();
        let res = _db.add_event(
            &message.chat.id.to_string(), 
            a[0].parse::<i32>().unwrap(),
            a[1]
        )
        .await?;

        bot.send_message(q.from.id, res).await?;
    }

    Ok(())
}

fn make_keyboard(companies: Vec<parser::CompanyInfo>) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    for company in companies {
        let info = format!("{}|{}", company.company_id, company.name);
        let button = InlineKeyboardButton::callback(
            company.name, info);

        keyboard.push(vec![ button ]);
    }

    InlineKeyboardMarkup::new(keyboard)
}