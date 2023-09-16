use super::{
    application::MailService,
    domain::Email,
    error::{Error, Result},
};
use crate::{on_error, smtp::Smtp, token::domain::Token};
use std::sync::Arc;
use tera::{Context, Tera};

pub struct UserSmtp<'a, E> {
    pub mta_mailer: Arc<E>,
    pub smtp: &'a Smtp<'a>,
    pub tera: &'a Tera,
    pub verification_subject: &'a str,
    pub verification_template: &'a str,
    pub reset_subject: &'a str,
    pub reset_template: &'a str,
}

impl<'a, E> MailService for UserSmtp<'a, E> {
    #[instrument(skip(self))]
    fn send_credentials_verification_email(&self, email: &Email, token: &Token) -> Result<()> {
        let mut context = Context::new();
        context.insert("name", email.username());
        context.insert("token", token.as_ref());

        let body = self
            .tera
            .render(self.verification_template, &context)
            .map_err(on_error!(
                Error,
                "rendering verification signup email template"
            ))?;

        self.smtp
            .send(email, self.verification_subject, &body)
            .map_err(Into::into)
    }

    #[instrument(skip(self))]
    fn send_credentials_reset_email(&self, email: &Email, token: &Token) -> Result<()> {
        let mut context = Context::new();
        context.insert("name", email.username());
        context.insert("token", token.as_ref());

        let body = self
            .tera
            .render(self.reset_template, &context)
            .map_err(on_error!(
                Error,
                "rendering verification reset email template"
            ))?;

        self.smtp
            .send(email, self.reset_subject, &body)
            .map_err(Into::into)
    }
}