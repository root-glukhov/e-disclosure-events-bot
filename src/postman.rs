use chrono::{Utc};
use dotenvy_macro::dotenv;
use teloxide::{Bot, requests::{Requester}, types::{ChatId, ParseMode}, payloads::SendMessageSetters};
use tokio::{time};

use crate::{DB, parser::{self, EventInfo}};


pub async fn start() {
    let bot = Bot::new(dotenv!("TELOXIDE_TOKEN"));

    let mut interval = time::interval(
        time::Duration::from_secs(
            dotenv!("POSTMAN_INTERVAL_SEC").parse::<u64>().unwrap()
        )
    );

    // Database
    let _db = DB.get().unwrap();

    loop {
        interval.tick().await;

        // Get Moscow Time
        let moscow_time = Utc::now().naive_utc() 
            + chrono::Duration::hours(3);

        let subscriptions = _db.get_subscriptions()
            .await
            .unwrap();

        for subscr in subscriptions {
            let events = parser::search_events(subscr.1)
                .await
                .unwrap()
                .into_iter()
                .filter(|x| {
                    x.date_time > moscow_time - chrono::Duration::seconds(
                        dotenv!("POSTMAN_INTERVAL_SEC").parse::<i64>().unwrap()
                    )
                })
                .collect::<Vec<EventInfo>>();

            for event in events {
                println!("{} - {}", event.date_time, event.url);

                let dt_str = event.date_time.format("%d.%m.%Y %H:%M").to_string();
                bot.send_message(ChatId(subscr.0), 
                    format!("<b>{}</b> {}\n<a href=\"{}\">{}</a>", 
                        subscr.2, 
                        dt_str, 
                        event.url, 
                        event.title
                    )
                ).parse_mode(ParseMode::Html)
                .disable_web_page_preview(true)
                .await
                .expect("Error send message");
            }
        }
    }
}