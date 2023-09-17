use std::sync::Arc;

use crate::{
    cache::Cache,
    mfa::{
        domain::Otp,
        error::{Error, Result},
        service::{MailService, MfaService},
    },
    secret::{
        application::SecretRepository,
        domain::{Secret, SecretKind},
    },
    user::domain::User,
};
use async_trait::async_trait;
use libreauth::oath::TOTP;
use std::time::Duration;

pub struct TpAppMethod<S, M, C> {
    pub otp_timeout: Duration,
    pub totp_secret_len: usize,
    pub secret_repo: Arc<S>,
    pub mail_srv: Arc<M>,
    pub cache: Arc<C>,
}

#[async_trait]
impl<S, M, C> MfaService for TpAppMethod<S, M, C>
where
    S: SecretRepository + Sync + Send,
    M: MailService + Sync + Send,
    C: Cache + Sync + Send,
{
    async fn verify(&self, user: &User, totp: Option<&Otp>) -> Result<()> {
        let totp = totp.ok_or(Error::Required)?;

        let actual_totp: TOTP = self
            .secret_repo
            .find_by_owner_and_kind(user.id, SecretKind::Otp)
            .await
            .map_err(Into::into)
            .and_then(TryInto::try_into)?;

        if actual_totp.is_valid(totp.as_ref()) {
            return Ok(());
        }

        Err(Error::Invalid)
    }

    async fn enable(&self, user: &User, totp: Option<&Otp>) -> Result<()> {
        let actual_otp = match self
            .cache
            .find::<Otp>(&Self::key(user))
            .await
            .map(Option::Some)
        {
            Err(err) if err.is_not_found() => Ok(None),
            other => other,
        }
        .map_err(Error::from)?;

        let Some(secret) = actual_otp else {
            let otp = self.totp_secret(user, self.totp_secret_len).await?;
            return Err(Error::Ack(AsRef::<str>::as_ref(&otp).to_string()));
        };

        let totp = totp.ok_or(Error::Required)?;

        let actual_totp: TOTP = secret.try_into()?;
        if !actual_totp.is_valid(totp.as_ref()) {
            return Err(Error::Invalid);
        }

        let mut secret = Secret::new(SecretKind::Otp, user, totp.as_ref().as_bytes());
        self.secret_repo
            .create(&mut secret)
            .await
            .map_err(Into::into)
    }

    async fn disable(&self, user: &User, totp: Option<&Otp>) -> Result<()> {
        let totp = totp.ok_or(Error::Required)?;

        let actual_totp: TOTP = self
            .secret_repo
            .find_by_owner_and_kind(user.id, SecretKind::Otp)
            .await
            .map_err(Into::into)
            .and_then(TryInto::try_into)?;

        if !actual_totp.is_valid(totp.as_ref()) {
            return Err(Error::Invalid);
        }

        let secret = self
            .secret_repo
            .find_by_owner_and_kind(user.id, SecretKind::Otp)
            .await?;

        self.secret_repo.delete(&secret).await.map_err(Into::into)
    }
}

impl<S, M, C> TpAppMethod<S, M, C>
where
    S: SecretRepository + Sync + Send,
    M: MailService + Sync + Send,
    C: Cache + Sync + Send,
{
    async fn totp_secret(&self, user: &User, len: usize) -> Result<Otp> {
        let otp = Otp::with_length(len)?;
        self.cache
            .save(&Self::key(user), &otp, self.otp_timeout)
            .await
            .map(|_| otp)
            .map_err(Into::into)
    }
}

impl<S, M, C> TpAppMethod<S, M, C> {
    fn key(user: &User) -> String {
        [&user.id.to_string(), "totp"].join("::")
    }
}