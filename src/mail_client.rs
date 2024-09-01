use anyhow::Result;
use futures::TryStreamExt;
use std::collections::HashMap;
use tokio::net::TcpStream;

const QUERY: &str = "(UID RFC822)";
const INBOX: &str = "INBOX";
const SUPPLIERS: [&str; 6] = [
    "vvolodin@opuscontract.ru",
    "sales@bratec-lis.com",
    "rassilka@fancyfloor.ru",
    "ulyana.boyko@carpetland.ru",
    "dealer@kover-zefir.ru",
    "almaz2008@yandex.ru",
];
pub async fn new(user: &str, pass: &str, host: &str) -> Result<MailClient> {
    let imap_addr = (host, 993);
    let tcp_stream = TcpStream::connect(imap_addr).await?;
    let tls = async_native_tls::TlsConnector::new().danger_accept_invalid_certs(true);
    let tls_stream = tls.connect(host, tcp_stream).await?;
    let client = async_imap::Client::new(tls_stream);
    let mut session = client.login(user, pass).await.map_err(|e| e.0)?;
    session.select(INBOX).await?;
    let msg_count = session.search("ALL").await?.len();
    let offset = msg_count - 100;
    session.logout().await?;
    Ok(MailClient {
        user: user.to_string(),
        pass: pass.to_string(),
        host: host.to_string(),
        port: 993,
        offset,
    })
}
pub struct MailClient {
    user: String,
    pass: String,
    host: String,
    port: u16,
    offset: usize,
}
impl MailClient {
    async fn session(&self) -> Result<async_imap::Session<async_native_tls::TlsStream<TcpStream>>> {
        let imap_addr = (self.host.clone(), self.port);
        let tcp_stream = TcpStream::connect(imap_addr).await?;
        let tls = async_native_tls::TlsConnector::new().danger_accept_invalid_certs(true);
        let tls_stream = tls.connect(&self.host, tcp_stream).await?;
        let client = async_imap::Client::new(tls_stream);
        let mut session = client
            .login(&self.user, &self.pass)
            .await
            .map_err(|e| e.0)?;
        session.select(INBOX).await?;
        Ok(session)
    }
    pub async fn fetch(&self) -> Result<HashMap<String, Vec<Vec<u8>>>> {
        let mut session = self.session().await?;
        let msg_count = session.search("ALL").await?.len();
        let q = format!("{}:{msg_count}", self.offset);
        let fetches = session
            .fetch(q, QUERY)
            .await?
            .try_collect::<Vec<_>>()
            .await?;
        let mut m = HashMap::new();
        for fetch in fetches {
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
                            .map(|a| a.contents().to_vec())
                            .collect::<Vec<_>>();
                        m.insert(sender, attachments);
                    }
                }
            }
        }
        session.logout().await?;
        Ok(m)
    }
}
