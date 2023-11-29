use lettre::transport::smtp::authentication::Credentials; 
use lettre::{Message, SmtpTransport, Transport, Address}; 
use lettre::message::Mailbox;
use crate::config::Config;

#[derive(Clone)]
pub struct Mailer {
    host: String,
    port: u16,
    auth_user: String,
    auth_pwd: String,
    app_name: String,
}

impl Mailer {
    pub fn new(config: &Config) -> Mailer {
        Mailer {
            host: config.mail_host.clone(),
            port: config.mail_port.clone(),
            auth_user: config.mail_auth_user.clone(),
            auth_pwd: config.mail_auth_pwd.clone(),
            app_name: config.app_name.clone()
        }
    }

    pub fn send_message(&self, receiver: String, subject: String, message: String) {
      let email = Message::builder() 
        .from(Mailbox::new(Some(self.app_name.clone()), self.auth_user.parse::<Address>().unwrap())) 
        .to(Mailbox::new(None, receiver.parse().unwrap())) 
        .subject(&subject) 
        .message_id(None)
        .body(message) 
        .unwrap(); 

      let creds = Credentials::new(self.auth_user.to_string(), self.auth_pwd.to_string()); 
      let mailer = SmtpTransport::starttls_relay(&self.host)
        .unwrap() 
        .port(self.port)
        .credentials(creds) 
        .build();

      match mailer.send(&email) { 
        Ok(_) => println!("Email sent successfully!"), 
        Err(e) => panic!("Could not send email: {:?}", e), 
      }
    }
   
}
