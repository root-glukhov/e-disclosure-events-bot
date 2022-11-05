use chrono::NaiveDateTime as DateTime;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};


#[derive(Debug)]
pub struct EventInfo {
    pub date_time: DateTime,
    pub title: String,
    pub url: String
}

fn http_client() -> Client {
    reqwest::Client::builder()
        .user_agent("e-disclosure events bot")
        .cookie_store(true)
        .build()
        .unwrap()
}

pub async fn search_company(query: &str) -> Result<Vec<(i32, String)>, String> {
    let params = [
        ("lastPageSize", "10"),
        ("lastPageNumber", "1"),
        ("districtsCheckboxGroup", "-1"),
        ("regionsCheckboxGroup", "-1"),
        ("branchesCheckboxGroup", "-1"),
        ("query", query),
    ];

    let url = "https://e-disclosure.ru/poisk-po-kompaniyam";
    let html = http_client().post(url)
        .form(&params)
        .send().await.unwrap()
        .text().await.unwrap();

    let td = Selector::parse("td").unwrap();
    let a = Selector::parse("a").unwrap();
    let re_i32 = Regex::new(r"\d+").unwrap();

    let result = Html::parse_fragment(&html)
        .select(&td)
        .collect::<Vec<_>>()
        .chunks_exact(6)
        .map(|chunk| {
            let link_elem = chunk[0].select(&a).next().unwrap();
            let link_href = link_elem.value().attr("href").unwrap();
            let company_id = String::from(
                re_i32.find(link_href).unwrap().as_str())
                .parse::<i32>().unwrap();

            (company_id, link_elem.inner_html())
        })
        .collect::<Vec<(i32, String)>>();

    Ok(result)
}

pub async fn search_events(company_id: i32) -> Result<Vec<EventInfo>, String> {
    let url = format!(
        "https://e-disclosure.ru/Event/Page?companyId={}&year=2022", 
        company_id
    );

    let html = http_client().get(url)
        .send().await.unwrap()
        .text().await.unwrap();

    let fragment = Html::parse_fragment(&html);

    let td = Selector::parse("td").unwrap();
    let a = Selector::parse("a").unwrap();

    Ok(fragment.select(&td)
        .collect::<Vec<_>>()
        .chunks_exact(3)
        .map(|chunk| {
            let link_elem = chunk[2].select(&a).next().unwrap();
            let link_href = link_elem.value().attr("href").unwrap();

            let dt_str = chunk[1].inner_html().replace("&nbsp;", " ");
            let dt = DateTime::parse_from_str(&dt_str, "%d.%m.%Y %H:%M")
                .unwrap();
                
            EventInfo {
                date_time: dt, 
                title: link_elem.inner_html(),
                url: link_href.to_string()
            }
        })
        .collect::<Vec<EventInfo>>())
}