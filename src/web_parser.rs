use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::info;

pub struct WebParser {
    username: String,
    password: String,
    client: reqwest::Client,
}
pub fn new(user: &str, pass: &str) -> Result<WebParser> {
    let username = user.to_string();
    let password = pass.to_string();
    let cookie_store = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let mut def_head = reqwest::header::HeaderMap::new();
    let v = reqwest::header::HeaderValue::from_str(
        "https://www.yandex.ru/clck/jsredir?from=yandex.ru;suggest;browser&text=",
    )?;
    def_head.insert(reqwest::header::REFERER, v);
    let client = reqwest::Client::builder()
        .gzip(true)
        .default_headers(def_head)
        .cookie_store(true)
        .cookie_provider(cookie_store.clone())
        .user_agent("Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36")
        .build()?;
    Ok(WebParser {
        username,
        password,
        client,
    })
}
impl WebParser {
    fn user(&self) -> &str {
        &self.username
    }
    fn pass(&self) -> &str {
        &self.password
    }
    pub async fn ortgraph(&self) -> Result<Vec<Vec<u8>>> {
        const BASE_URI: &str = "https://ortgraph.ru";
        const STOCK: &str = "remains/";
        const AUTH: &str = "auth/";
        const PERSONAL: &str = "/personal/";
        let mut auth_uri = BASE_URI.to_string();
        auth_uri.push('/');
        auth_uri.push_str(AUTH);
        let mut form = HashMap::new();
        form.insert("AUTH_FORM", "Y");
        form.insert("TYPE", "AUTH");
        form.insert("backurl", "/auth/");
        form.insert("USER_LOGIN", self.user());
        form.insert("USER_PASSWORD", self.pass());
        form.insert("Login", "Войти");
        let response = self
            .client
            .post(&auth_uri)
            .query(&[("login", "yes")])
            .form(&form)
            .send()
            .await?;
        if response.status() != reqwest::StatusCode::OK {
            let b = response.text().await?;
            info!("Something with auth!!!!!!!!!!!");
            info!("{b:#?}");
        }
        let mut stock_uri = BASE_URI.to_string();
        stock_uri.push_str(PERSONAL);
        stock_uri.push_str(STOCK);
        let response = self
            .client
            .get(&stock_uri)
            .query(&[("login", "yes")])
            .send()
            .await?;
        let body = response.text().await?;
        let links = get_links(body);
        let mut files = Vec::new();
        for path in links {
            let mut uri = BASE_URI.to_string();
            uri.push_str(&path);
            let temp_res = self.client.get(&uri).send().await?;
            let bytes = temp_res.bytes().await?.as_ref().to_vec();
            files.push(bytes.to_owned());
        }
        Ok(files)
    }
    pub async fn vvk(&self) -> Result<Vec<Vec<u8>>> {
        const BASE_URI: &str = "https://disk.yandex.ru/d/1qA555p_DbQiaQ";
        let uri = "https://cloud-api.yandex.net:443/v1/disk/public/resources";
        // let download_uri = "https://cloud-api.yandex.net:443/v1/disk/public/resources/download";
        let mut result = Vec::new();
        let response = self.client
            .get(uri)
            .query(&[("public_key", BASE_URI)])
            .send()
            .await?;
        let root: Root = response.json().await?;
        for i in root.embedded.items {
            // info!("Got vvk link: {}", i.file);
            let file = self.client.get(i.file).send().await?.bytes().await?.to_vec();
            result.push(file)
        }
        Ok(result)
    }
}
fn get_links(body: String) -> Vec<String> {
    let mut result = Vec::new();
    let dom = tl::parse(&body, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let links = dom.query_selector("a[href]").unwrap();
    for link in links {
        let tag = link.get(parser).unwrap().as_tag().unwrap();
        let l = tag
            .attributes()
            .get("href")
            .flatten()
            .unwrap()
            .as_utf8_str()
            .to_string();
        let annotation = tag.inner_html(parser);
        if l.contains(".xls")
            && l.contains("upload")
            && (annotation.to_lowercase().contains("ковр")
            || annotation.to_lowercase().contains("напол"))
        {
            // info!("Got link for {annotation}: {l}");
            result.push(l)
        }
    }
    result
}
#[derive(Deserialize)]
struct Root {
    #[serde(rename = "_embedded")]
    pub embedded: Embedded,
}
#[derive(Deserialize)]
struct Embedded {
    items: Vec<Item>,
}
#[derive(Deserialize)]
struct Item {
    file: String,
}
