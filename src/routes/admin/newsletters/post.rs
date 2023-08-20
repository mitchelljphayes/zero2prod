//! src/routes/admin/newsletters/post.rs

use crate::authentication::UserId;
use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::routes::error_chain_fmt;
use crate::utils::see_other;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::PgPool;

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
            PublishError::AuthError(_) => HttpResponse::new(StatusCode::UNAUTHORIZED),
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct NewsletterFormData {
    pub newsletter_title: String,
    pub newsletter_html: String,
    pub newsletter_plain_text: String,
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(form, pool, email_client),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    form: web::Form<NewsletterFormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, PublishError> {
    tracing::Span::current().record("user_id", &tracing::field::display(*user_id));
    let subscribers = get_confirmed_subscribers(&pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &form.0.newsletter_title,
                        &form.0.newsletter_html,
                        &form.0.newsletter_plain_text,
                    )
                    .await
                    .with_context(|| {
                        FlashMessage::error(format!(
                            "Failed to send newsletter to {}",
                            &subscriber.email
                        ))
                        .send();
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })?;
                FlashMessage::info(format!("Email sent to {}", &subscriber.email)).send();
            }
            Err(error) => {
                FlashMessage::error("Some subscibers details are invalid").send();
                tracing::warn!(
                    error.cause_chain = ?error,
                    "\nSkipping a confirmed subscriber. \
                    Their stored contact details are invalid."
                )
            }
        }
    }
    FlashMessage::info("Newsletter sent").send();
    Ok(see_other("/admin/newsletters"))
    // Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();
    Ok(confirmed_subscribers)
}
