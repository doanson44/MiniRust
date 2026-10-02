use std::sync::Arc;

use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use minirust_config::SmtpConfig;
use minirust_services::{AuthError, ChallengePurpose, EmailSender};

#[derive(Clone)]
pub struct SmtpEmailSender {
    transport: Option<Arc<AsyncSmtpTransport<Tokio1Executor>>>,
    from: Option<Mailbox>,
}

impl SmtpEmailSender {
    pub fn from_config(config: Option<&SmtpConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let Some(config) = config else {
            return Ok(Self {
                transport: None,
                from: None,
            });
        };

        let from = match &config.from_name {
            Some(name) => format!("{} <{}>", name, config.from_email).parse()?,
            None => config.from_email.parse()?,
        };

        let credentials = Credentials::new(config.username.clone(), config.password.clone());
        let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)?
            .port(config.port)
            .credentials(credentials)
            .build();

        Ok(Self {
            transport: Some(Arc::new(transport)),
            from: Some(from),
        })
    }

    async fn send(&self, recipient: &str, subject: &str, body: String) -> Result<(), AuthError> {
        let (Some(transport), Some(from)) = (&self.transport, &self.from) else {
            return Err(AuthError::EmailDeliveryUnavailable);
        };

        let to = recipient
            .parse()
            .map_err(|_| AuthError::InvalidEmail)?;

        let message = Message::builder()
            .from(from.clone())
            .to(to)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)
            .map_err(|_| AuthError::EmailDeliveryUnavailable)?;

        transport
            .send(message)
            .await
            .map(|_| ())
            .map_err(|error| {
                tracing::error!(%error, "SMTP email delivery failed");
                AuthError::EmailDeliveryUnavailable
            })
    }
}

impl EmailSender for SmtpEmailSender {
    async fn send_verification_code(
        &self,
        email: &str,
        purpose: ChallengePurpose,
        code: &str,
    ) -> Result<(), AuthError> {
        let purpose_text = match purpose {
            ChallengePurpose::Registration => "registration",
            ChallengePurpose::Login => "login",
        };

        self.send(
            email,
            "MiniRust verification code",
            format!(
                "Your MiniRust {purpose_text} verification code is: {code}\n\nThis code expires in 10 minutes."
            ),
        )
        .await
    }

    async fn send_registration_verification(
        &self,
        email: &str,
        token: &str,
    ) -> Result<(), AuthError> {
        self.send(
            email,
            "MiniRust registration verification",
            format!(
                "Your MiniRust registration verification token is:\n\n{token}\n\nThis token expires in 10 minutes."
            ),
        )
        .await
    }
}
