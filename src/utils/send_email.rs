use crate::config::mail_config::MailService;
use lettre::{
    AsyncTransport, Message,
    message::{Mailbox, header::ContentType},
};

impl MailService {
    pub async fn send_otp_email(
        &self,
        to: &str,
        username: &str,
        otp: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let html = format!(
            r#"
            <html>
            <body style="font-family:Arial">
                <h2>Email Verification</h2>

                <p>Hello {},</p>

                <p>Your OTP is:</p>

                <div style="
                    font-size:32px;
                    font-weight:bold;
                    letter-spacing:8px;
                    padding:20px;
                    background:#f3f4f6;
                    text-align:center;
                    border-radius:8px;
                ">
                    {}
                </div>

                <p>This OTP expires in 15 minutes.</p>
            </body>
            </html>
            "#,
            username, otp
        );

        let sender = std::env::var("gmail_email")
            .or_else(|_| std::env::var("SMTP_USER"))
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let email = Message::builder()
            .from(Mailbox::new(Some("Your App".to_string()), sender.parse()?))
            .to(to.parse()?)
            .subject("Verify Your Email")
            .header(ContentType::TEXT_HTML)
            .body(html)?;

        self.mailer.send(email).await?;

        Ok(())
    }
}
