use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::Error;

pub fn send_email(recipient: &str, subject: &str, body: &str) -> Result<(), Error> {

  let username = std::env::var("SMTP_USERNAME").expect("Could not find SMTP_USERNAME in environment variables");
  let password = std::env::var("SMTP_PASSWORD").expect("Could not find SMTP_PASSWORD in environment variables");
  let sender = std::env::var("SMTP_EMAIL").expect("Could not find SMTP_EMAIL in environment variables");
  let server = std::env::var("SMTP_SERVER").expect("Could not find SMTP_SERVER in environment variables");

  let email = Message::builder()
    .from(sender.parse()?)
    .to(recipient.parse()?)
    .subject(subject)
    .body(body.to_string())
    ?;

  let creds = Credentials::new(username.to_string(), password.to_string());

  let mailer = SmtpTransport::relay(server.as_str())?.credentials(creds).build();

  match mailer.send(&email) {
      Ok(_) => Ok(()),
      Err(e) => {
        Err(Box::new(e))
      },
  }

}