use std::error::Error;
use std::time::Duration;
use crate::metadata::domain::Metadata;
use crate::security;
use crate::constants::{errors, settings};
use crate::smtp;
use crate::session::{
    get_repository as get_sess_repository,
    application::get_writable_session,
};
use crate::directory::get_repository as get_dir_repository;
use super::domain::{User, Token};

pub fn user_signup(email: &str,
                   password: &str) -> Result<(), Box<dyn Error>> {
    
    info!("got signup request from user {} ", email);
    
    let meta = Metadata::new()?;
    let user = User::new(meta, email, password)?;
    
    // the user will not be able to log in until they have verified their email
    let claim = Token::new(&user, Duration::from_secs(settings::TOKEN_TIMEOUT));
    let token = security::encode_jwt(claim)?;
    smtp::send_verification_email(email, &token)?;

    Ok(())
}

pub fn user_delete(email: &str,
                   pwd: &str,
                   totp: &str) -> Result<(), Box<dyn Error>> {
    
    info!("got a deletion request from user {} ", email);

    let user = super::get_repository().find(email)?;
    if !user.match_password(pwd) {
        return Err(errors::NOT_FOUND.into());
    }

    // if, and only if, the user has activated the 2fa
    if let Some(secret) = &user.secret {
        let data = secret.get_data();
        security::verify_totp(data, totp)?;
    }

    // if the user was logged in the session must be removed
    if let Ok(sess_arc) = get_sess_repository().find_by_email(&user.email) {
        let mut sess = get_writable_session(&sess_arc)?;
        sess.delete(false)?; // do not save directories
    }

    // delete all directories
    get_dir_repository().delete_all_by_user(&user)?;
    
    user.delete()?;
    Ok(())
}