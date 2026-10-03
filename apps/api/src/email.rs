use std::sync::Arc;

use anyhow::Context;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{MultiPart, SinglePart, header},
    transport::smtp::authentication::Credentials,
};
use minirust_services::auth::{AuthError, ChallengePurpose, EmailSender};
use tokio::sync::mpsc;

use crate::config::EmailConfig;

const EMAIL_QUEUE_CAPACITY: usize = 64;

#[derive(Debug, Clone)]
struct EmailMessage {
    recipient: String,
    subject: String,
    body: String,
}

#[derive(Clone)]
pub struct SmtpEmailSender {
    tx: mpsc::Sender<EmailMessage>,
    web_url: String,
}

impl SmtpEmailSender {
    pub fn new(config: &EmailConfig) -> anyhow::Result<Self> {
        let credentials = Credentials::new(config.username.clone(), config.password.clone());
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)?
            .port(config.port)
            .credentials(credentials)
            .build();

        let (tx, mut rx) = mpsc::channel::<EmailMessage>(EMAIL_QUEUE_CAPACITY);
        let sender = Self {
            tx,
            web_url: config.web_url.clone(),
        };

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(error) = send_message(&transport, &message).await {
                    tracing::error!(%error, recipient = %message.recipient, "background email delivery failed");
                } else {
                    tracing::info!(recipient = %message.recipient, "background email delivered");
                }
            }
        });

        Ok(sender)
    }

    async fn enqueue(&self, recipient: &str, subject: &str, body: String) -> Result<(), AuthError> {
        self.tx
            .send(EmailMessage {
                recipient: recipient.to_owned(),
                subject: subject.to_owned(),
                body,
            })
            .await
            .map_err(|_| AuthError::EmailDeliveryFailed)
    }
}

async fn send_message(
    transport: &AsyncSmtpTransport<Tokio1Executor>,
    message: &EmailMessage,
) -> anyhow::Result<()> {
    let from = std::env::var("MINIRUST_EMAIL_FROM")
        .context("MINIRUST_EMAIL_FROM is required when email is enabled")?;

    let message = Message::builder()
        .from(from.parse()?)
        .to(message.recipient.parse()?)
        .subject(&message.subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(message.body.clone()),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(message.body.clone()),
                ),
        )?;

    transport.send(message).await?;
    Ok(())
}

impl EmailSender for SmtpEmailSender {
    async fn send_verification_code(
        &self,
        email: &str,
        purpose: ChallengePurpose,
        code: &str,
    ) -> Result<(), AuthError> {
        let _ = purpose; // Purpose can be ignored in the general template, or we could have multiple templates
        let subject = "[MiniRust] Verifying it's you / Xác minh danh tính";
        let body_template = include_str!("../templates/email_verification_code.html");
        let body = body_template.replace("{code}", code);
        self.enqueue(email, subject, body)
    }

    async fn send_invitation_link(&self, email: &str, token: &str) -> Result<(), AuthError> {
        let base_url = self.web_url.trim_end_matches('/');
        let link = format!("{base_url}/invite/accept?token={token}");
        let subject = "[MiniRust] Bạn được mời / You are invited";
        let body_template = include_str!("../templates/email_invitation.html");
        let body = body_template.replace("{link}", &link);
        self.enqueue(email, subject, body)
    }

    async fn send_registration_verification(
        &self,
        email: &str,
        token: &str,
    ) -> Result<(), AuthError> {
        let base_url = self.web_url.trim_end_matches('/');
        let link = format!("{base_url}/register/verify?token={token}");
        let subject = "[MiniRust] Verify your email / Xác minh email";
        let body_template = include_str!("../templates/email_registration_verification.html");
        let body = body_template.replace("{link}", &link);
        self.enqueue(email, subject, body)
    }
}
