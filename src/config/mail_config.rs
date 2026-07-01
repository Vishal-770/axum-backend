use lettre::{AsyncSmtpTransport, Tokio1Executor, transport::smtp::authentication::Credentials};

#[derive(Clone, Debug)]
pub struct MailService {
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl MailService {
    pub fn new() -> Self {
        let smtp_user = std::env::var("gmail_email")
            .or_else(|_| std::env::var("SMTP_USER"))
            .expect("gmail_email or SMTP_USER must be set");
        let smtp_pass = std::env::var("gmail_app_password")
            .or_else(|_| std::env::var("SMTP_PASS"))
            .expect("gmail_app_password or SMTP_PASS must be set");

        let creds = Credentials::new(smtp_user, smtp_pass);

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

        Self { mailer }
    }
}

impl Default for MailService {
    fn default() -> Self {
        Self::new()
    }
}
