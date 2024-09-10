use anyhow::{anyhow, Result};
use mail_parser::MimeHeaders;
use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::ops::Deref;

const QUERY: &str = "RFC822";
const INBOX: &str = "INBOX";
const SUPPLIERS: [&str; 7] = [
    "vvolodin@opuscontract.ru",
    "sales@bratec-lis.com",
    "rassilka@fancyfloor.ru",
    "sale8@fancy-floor.ru",
    "ulyana.boyko@carpetland.ru",
    "dealer@kover-zefir.ru",
    "almaz2008@yandex.ru",
];
pub fn new(user: &str, pass: &str, host: &str) -> Result<MailClient> {
    let socket_addr = format!("{host}:993")
        .to_socket_addrs()?
        .next()
        .ok_or(anyhow!("Wrong host or port"))?;
    let mut mail_client = MailClient {
        user: user.to_string(),
        pass: pass.to_string(),
        host: host.to_string(),
        socket: socket_addr,
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
    socket: SocketAddr,
    from: usize,
}
impl MailClient {
    fn session(&self) -> Result<imap::Session<native_tls::TlsStream<std::net::TcpStream>>> {
        let tls = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
        let client = imap::connect(self.socket, &self.host, &tls)?;
        let mut session = client.login(&self.user, &self.pass).map_err(|e| e.0)?;
        session.select(INBOX)?;
        Ok(session)
    }
    pub fn fetch(&mut self) -> Result<HashMap<String, Vec<Vec<u8>>>> {
        let mut session = self.session()?;
        let msg_count = session.search("ALL")?.len();
        let q = format!("{}:{msg_count}", self.from);
        let fetches = session.fetch(q, QUERY)?;
        self.from = msg_count;
        let mut m = HashMap::new();
        for fetch in fetches.deref() {
            if let Some(body) = fetch.body() {
                if let Some(parsed) = mail_parser::MessageParser::default().parse(body) {
                    let sender = parsed
                        .from()
                        .and_then(|a| a.first().and_then(|s| s.address()))
                        .map(|s| s.to_lowercase())
                        .unwrap_or_default();
                    if SUPPLIERS.contains(&sender.as_str()) {
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
                            m.insert(sender, attachments);
                        }
                    }
                }
            }
        }
        session.logout()?;
        Ok(m)
    }
}
