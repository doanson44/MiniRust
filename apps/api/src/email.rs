use std::sync::Arc;

use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use minirust_config::Config;
use minirust_services::{AuthError, ChallengePurpose, EmailSender};

#[derive(Clone)]
pub struct SmtpEmailSender {
    transport: Option<Arc<AsyncSmtpTransport<Tokio1Executor>>>,
    from: Option<Mailbox>,
    local: bool,
    web_url: String,
}

impl SmtpEmailSender {
    pub fn disabled(web_url: String) -> Self {
        Self {
            transport: None,
            from: None,
            local: false,
            web_url,
        }
    }

    /// Creates an in-memory sender for integration tests; it never contacts SMTP.
    pub fn local_for_tests(web_url: String) -> Self {
        Self {
            transport: None,
            from: None,
            local: true,
            web_url,
        }
    }

    pub fn from_config(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let Some(smtp) = &config.smtp else {
            return Ok(Self::disabled(config.web_url.clone()));
        };
        let from = match &smtp.from_name {
            Some(name) => format!("{name} <{}>", smtp.from_email).parse()?,
            None => smtp.from_email.parse()?,
        };
        let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host)?
            .port(smtp.port)
            .credentials(Credentials::new(
                smtp.username.clone(),
                smtp.password.clone(),
            ))
            .build();
        Ok(Self {
            transport: Some(Arc::new(transport)),
            from: Some(from),
            local: false,
            web_url: config.web_url.clone(),
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.transport.is_some() && self.from.is_some()
    }

    /// Queues delivery on a background task; the result only reports configuration
    /// and recipient validation, never whether the message was delivered.
    fn enqueue(&self, recipient: &str, subject: &str, body: String) -> Result<(), AuthError> {
        if self.local {
            tracing::info!(recipient = %recipient, subject, "local test email delivery simulated");
            return Ok(());
        }
        let (Some(transport), Some(from)) = (self.transport.clone(), self.from.clone()) else {
            return Err(AuthError::EmailDeliveryUnavailable);
        };
        let to = recipient.parse().map_err(|_| AuthError::InvalidEmail)?;
        let message = Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .map_err(|_| AuthError::EmailDeliveryUnavailable)?;

        let recipient = recipient.to_owned();
        tokio::spawn(async move {
            match transport.send(message).await {
                Ok(_) => tracing::info!(recipient = %recipient, "background email delivered"),
                Err(error) => {
                    tracing::error!(%error, recipient = %recipient, "background email delivery failed")
                }
            }
        });

        Ok(())
    }
}

impl EmailSender for SmtpEmailSender {
    async fn send_verification_code(
        &self,
        email: &str,
        purpose: ChallengePurpose,
        code: &str,
    ) -> Result<(), AuthError> {
        let _ = purpose; // Purpose can be ignored in the general template, or we could have multiple templates
        let subject = format!("[MiniRust] Mã xác minh / Verification code - {code}");
        let body_template = include_str!("../templates/email_verification_code.html");
        let body = body_template.replace("{code}", code);
        self.enqueue(email, &subject, body)
    }

    async fn send_registration_verification(
        &self,
        email: &str,
        token: &str,
    ) -> Result<(), AuthError> {
        let link = format!("{}/register/verify?token={}", self.web_url, token);
        let subject = "[MiniRust] Hoàn tất đăng ký / Complete registration";
        let body_template = include_str!("../templates/email_registration_verification.html");
        let body = body_template.replace("{link}", &link);
        self.enqueue(email, subject, body)
    }
}
