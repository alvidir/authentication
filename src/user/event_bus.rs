use super::{application::EventBus, domain::User, error::Result};
use crate::{on_error, rabbitmq::EventKind};
use async_trait::async_trait;
use deadpool_lapin::Pool;
use lapin::{options::*, BasicProperties};
use serde_json;

#[derive(Serialize, Deserialize)]
struct UserEventPayload<'a> {
    pub(super) user_id: i32,
    pub(super) user_name: &'a str,
    pub(super) user_email: &'a str,
    pub(super) event_issuer: &'a str,
    pub(super) event_kind: EventKind,
}

pub struct RabbitMqUserBus<'a> {
    pub pool: &'a Pool,
    pub exchange: &'a str,
    pub issuer: &'a str,
}

#[async_trait]
impl<'a> EventBus for RabbitMqUserBus<'a> {
    #[instrument(skip(self))]
    async fn emit_user_created(&self, user: &User) -> Result<()> {
        let event = UserEventPayload {
            user_id: user.id,
            user_name: user.credentials.email.username(),
            user_email: user.credentials.email.as_ref(),
            event_issuer: self.issuer,
            event_kind: EventKind::Created,
        };

        let payload = serde_json::to_string(&event)
            .map(|str| str.into_bytes())
            .map_err(on_error!("serializing user created event data to json"))?;

        let connection = self
            .pool
            .get()
            .await
            .map_err(on_error!("pulling connection from rabbitmq pool"))?;

        connection
            .create_channel()
            .await
            .map_err(on_error!("creating rabbitmq channel"))?
            .basic_publish(
                self.exchange,
                "",
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await
            .map_err("emititng user created event")?
            .await
            .map_err("confirming user created event reception")?;

        Ok(())
    }
}
