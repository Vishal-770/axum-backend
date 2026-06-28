use sqlx::PgPool;
use crate::config::mail_config::MailService;

#[derive(Clone, Debug)]
pub struct AppState {
  pub db: PgPool,
  pub mail_service: MailService,
  pub redis: redis::aio::MultiplexedConnection,
}