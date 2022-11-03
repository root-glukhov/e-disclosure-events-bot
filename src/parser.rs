use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};

#[derive(Debug)]
pub struct CompanyInfo {
    pub company_id: i64,
    pub name: String
}

fn http_client() -> Client {
    reqwest::Client::builder()
        .user_agent("e-disclosure events bot")
        .cookie_store(true)
        .build()
        .unwrap()
}

pub async fn search_company(query: &str) -> Result<Vec<CompanyInfo>, String> {
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
    let re_i64 = Regex::new(r"\d+").unwrap();

    Ok(Html::parse_fragment(&html)
        .select(&td)
        .collect::<Vec<_>>()
        .chunks_exact(6)
        .map(|chunk| {
            let link_elem = chunk[0].select(&a).next().unwrap();
            let link_href = link_elem.value().attr("href").unwrap();
            let company_id = String::from(
                re_i64.find(link_href).unwrap().as_str())
                .parse::<i64>().unwrap();

            CompanyInfo {
                company_id: company_id,
                name: link_elem.inner_html()
            }
        })
        .collect::<Vec<CompanyInfo>>())
}