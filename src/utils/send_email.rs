use crate::config::mail_config::MailService;
use lettre::{
    message::{header::ContentType, Mailbox},
    AsyncTransport,
    Message,
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

                <p>This OTP expires in 10 minutes.</p>
            </body>
            </html>
            "#,
            username, otp
        );

        let email = Message::builder()
            .from(
                Mailbox::new(
                    Some("Your App".to_string()),
                    std::env::var("SMTP_USER")?.parse()?,
                )
            )
            .to(to.parse()?)
            .subject("Verify Your Email")
            .header(ContentType::TEXT_HTML)
            .body(html)?;

        self.mailer.send(email).await?;

        Ok(())
    }

    pub async fn send_password_reset_email(
        &self,
        to: &str,
        username: &str,
        otp: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let html = format!(
            r#"
            <html>
            <body style="font-family:Arial">
                <h2>Password Reset Request</h2>

                <p>Hello {},</p>

                <p>You requested a password reset. Your 6-digit OTP code is:</p>

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

                <p>This code expires in 10 minutes. If you did not request this, please ignore this email.</p>
            </body>
            </html>
            "#,
            username, otp
        );

        let email = Message::builder()
            .from(
                Mailbox::new(
                    Some("Your App".to_string()),
                    std::env::var("SMTP_USER")?.parse()?,
                )
            )
            .to(to.parse()?)
            .subject("Password Reset OTP")
            .header(ContentType::TEXT_HTML)
            .body(html)?;

        self.mailer.send(email).await?;

        Ok(())
    }
}
