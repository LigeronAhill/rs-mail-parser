use anyhow::Result;
use mail_parser::MimeHeaders;
use std::collections::HashMap;
use surrealdb::sql::Datetime;
use tracing::info;

const QUERY: &str = "RFC822";
const INBOX: &str = "INBOX";
pub fn new(user: &str, pass: &str, host: &str) -> Result<MailClient> {
    let mut mail_client = MailClient {
        user: user.to_string(),
        pass: pass.to_string(),
        host: host.to_string(),
        from: 0,
    };
    let mut session = mail_client.session()?;
    session.select(INBOX)?;
    let msg_count = session.search("ALL")?.len();
    mail_client.from = msg_count - 300;
    session.logout()?;
    Ok(mail_client)
}
#[derive(Clone)]
pub struct MailClient {
    user: String,
    pass: String,
    host: String,
    // socket: SocketAddr,
    from: usize,
}
impl MailClient {
    fn session(&self) -> Result<imap::Session<Box<dyn imap::ImapConnection>>> {
        let client = imap::ClientBuilder::new(&self.host, 993)
            .danger_skip_tls_verify(true)
            .connect()?;
        let mut session = client.login(&self.user, &self.pass).map_err(|e| e.0)?;
        session.select(INBOX)?;
        Ok(session)
    }
    pub fn fetch(&mut self) -> Result<HashMap<String, (Vec<Vec<u8>>, Datetime)>> {
        let mut supmap = HashMap::new();
        supmap.insert("vvolodin@opuscontract.ru", "opus");
        supmap.insert("sales@bratec-lis.com", "fox");
        supmap.insert("rassilka@fancyfloor.ru", "fancy");
        // supmap.insert("sale8@fancy-floor.ru", "fancy");
        supmap.insert("ulyana.boyko@carpetland.ru", "carpetland");
        supmap.insert("dealer@kover-zefir.ru", "zefir");
        supmap.insert("almaz2008@yandex.ru", "fenix");
        let mut session = self.session()?;
        let msg_count = session.search("ALL")?.len();
        let q = format!("{}:{msg_count}", self.from);
        info!("Fetching from {} to {msg_count}", self.from);
        let fetches = session.fetch(q, QUERY)?;
        self.from = msg_count;
        let mut m = HashMap::new();
        for fetch in fetches.iter() {
            if let Some(body) = fetch.body() {
                if let Some(parsed) = mail_parser::MessageParser::default().parse(body) {
                    let sender = parsed
                        .from()
                        .and_then(|a| a.first().and_then(|s| s.address()))
                        .map(|s| s.to_lowercase())
                        .unwrap_or_default();
                    if let Some(supplier) = supmap.get(sender.as_str()) {
                        let attachments = parsed
                            .attachments()
                            .flat_map(|a| {
                                if a.attachment_name().is_some_and(|n| {
                                    n.to_lowercase().contains("склад")
                                        || n.to_lowercase().contains("остат")
                                }) || (sender == "vvolodin@opuscontract.ru"
                                    && a.attachment_name().is_none())
                                {
                                    Some(a.contents().to_vec())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        if !attachments.is_empty() {
                            let received = if let Some(r) = parsed.date().and_then(|d| {
                                <Datetime as std::str::FromStr>::from_str(&d.to_rfc3339()).ok()
                            }) {
                                r
                            } else {
                                Datetime(chrono::Utc::now())
                            };
                            m.insert(supplier.to_string(), (attachments, received));
                        }
                    }
                }
            }
        }
        session.logout()?;
        Ok(m)
    }
}
