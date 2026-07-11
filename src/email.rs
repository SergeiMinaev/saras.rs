use crate::conf::CONF;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{Mailbox, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;

pub async fn send_plain_email(to: &str, subject: &str, body: &str) -> Result<(), String> {
	let conf = CONF.read().await;
	if conf.smtp_fake {
		println!("=== FAKE EMAIL ===\nTo: {to}\nSubject: {subject}\n\n{body}\n==================");
		return Ok(());
	}
	let msg = Message::builder()
		.from(conf.smtp_login.parse::<Mailbox>().map_err(|e| e.to_string())?)
		.to(to.parse::<Mailbox>().map_err(|e| e.to_string())?)
		.subject(subject)
		.header(ContentType::TEXT_PLAIN)
		.body(body.to_string())
		.map_err(|e| e.to_string())?;

	let creds = Credentials::new(conf.smtp_login.clone(), conf.smtp_pwd.clone());
	let mailer = SmtpTransport::relay(&conf.smtp_server)
		.map_err(|e| e.to_string())?
		.credentials(creds)
		.build();

	mailer.send(&msg).map_err(|e| e.to_string())?;
	Ok(())
}
