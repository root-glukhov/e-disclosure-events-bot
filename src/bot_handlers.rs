use crate::{DB, parser};

use teloxide::{
    utils::command::BotCommands, 
    Bot, types::{Message, Me, InlineKeyboardMarkup, InlineKeyboardButton, CallbackQuery}, 
    requests::Requester, payloads::SendMessageSetters
};


#[derive(BotCommands)]
#[command(rename_rule = "lowercase", description = "Команды:")]
enum Command {
    #[command(description = "Помощь v0.1")]
    Help,
    #[command(description = "/add <название компании> - чтобы добавить к отслеживаемым")]
    Add(String),
    #[command(description = "/delete - чтобы удалить компанию из отслеживаемых")]
    Delete,
}

struct Data {
    int: i32,
    text: String
}

impl From<(i32, String)> for Data {
    fn from(item: (i32, String)) -> Self {
        Data { int: item.0, text: item.1 }
    }
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
                let companies = parser::search_company(&query)
                    .await
                    .unwrap()
                    .iter()
                    .map(|x| {
                        Data { int: x.0, text: x.1.to_string() }
                    })
                    .collect::<Vec<Data>>();

                let keyboard = make_keyboard(
                    "add", companies
                );
                
                bot.send_message(msg.chat.id, "Выберите компанию из найденных:")
                    .reply_markup(keyboard)
                    .await?;
            }

            Ok(Command::Delete) => {
                let _db = DB.get().unwrap();
                let data =_db.get_events(
                    &msg.chat.id.to_string()
                )
                .await
                .unwrap()
                .iter()
                .map(|x| {
                    Data { int: x.0, text: x.1.to_string() }
                })
                .collect::<Vec<Data>>();

                let keyboard = make_keyboard("delete", data);
                bot.send_message(msg.chat.id, "Удаление из отслеживаемых:")
                    .reply_markup(keyboard)
                    .await?;
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


        let data: Vec<&str> = data.split("|").collect();
        let _db = DB.get().unwrap();
        let id = data[1].parse::<i32>().unwrap();

        match data[0] {
            "add" => {  
                let res = _db.add_event(
                    &message.chat.id.to_string(), 
                    id,
                    data[2]
                )
                .await;

                match res {
                    Ok(r) => bot.send_message(q.from.id, r).await?,
                    Err(e) => bot.send_message(q.from.id, format!("Ошибка: {}", e.to_string())).await?
                };
            },
            "delete" => {
                if let Ok(_) = _db.delete_event(id).await {
                    let res = format!(
                        "Компания {} удалена из отслеживаемых",
                        data[2]
                    );
                    
                    bot.send_message(q.from.id, res).await?;
                }
            },
            _ => { }
        }
    }
    Ok(())
}

fn make_keyboard(
    cmd: &str,
    data: Vec<Data>
) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    for item in data {
        let values = format!("{}|{}|{}", cmd, item.int, item.text);
        let button = InlineKeyboardButton::callback(
            item.text, values
        );

        keyboard.push(vec![ button ]);
    }

    InlineKeyboardMarkup::new(keyboard)
}