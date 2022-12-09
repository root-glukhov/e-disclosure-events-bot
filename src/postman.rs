use chrono::{Utc};
use dotenvy_macro::dotenv;
use teloxide::{Bot, requests::{Requester}, types::{ChatId, ParseMode}, payloads::SendMessageSetters};
use tokio::{time};

use crate::{DB, parser::{self, EventInfo}};


const INTERVAL: u64 = 60;

pub async fn start() {
    let bot = Bot::new(dotenv!("TELOXIDE_TOKEN"));

    let mut interval = time::interval(
        time::Duration::from_secs(INTERVAL)
    );

    // Database
    let _db = DB.get().unwrap();

    loop {
        interval.tick().await;

        // Get Moscow Time
        let moscow_time = Utc::now().naive_utc() + chrono::Duration::hours(3);
        let subs = _db.get_subs().await.unwrap();

        for sub in subs {
            let events = parser::search_events(sub.1)
                .await
                .unwrap()
                .into_iter()
                .filter(|x| {
                    x.date_time > moscow_time - chrono::Duration::seconds(INTERVAL as i64)
                })
                .collect::<Vec<EventInfo>>();

            for event in events {
                println!("{} - {}", event.date_time, event.url);

                let dt_str = event.date_time.format("%d.%m.%Y %H:%M").to_string();
                bot.send_message(ChatId(sub.0), 
                    format!("<b>{}</b> {}\n<a href=\"{}\">{}</a>", 
                        sub.2, 
                        dt_str, 
                        event.url, 
                        event.title
                    )
                )
                .parse_mode(ParseMode::Html)
                .disable_web_page_preview(true)
                .await
                .expect("Error send message");
            }
        }
    }
}