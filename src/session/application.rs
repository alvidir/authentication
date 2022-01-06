use std::time::Duration;
use std::error::Error;
use std::sync::Arc;
use super::domain::SessionToken;
use crate::user::application::UserRepository;
use crate::secret::application::SecretRepository;
use crate::regex;
use crate::constants;
use crate::security;

pub trait SessionRepository {
    fn find(&self, user_id: i32) -> Result<SessionToken, Box<dyn Error>>;
    fn save(&self, token: &SessionToken) -> Result<(), Box<dyn Error>>;
    fn delete(&self, user_id: i32) -> Result<(), Box<dyn Error>>;
}

pub struct SessionApplication<S: SessionRepository, U: UserRepository, E: SecretRepository> {
    pub session_repo: Arc<S>,
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<E>,
    pub lifetime: u64,
}

impl<S: SessionRepository, U: UserRepository, E: SecretRepository> SessionApplication<S, U, E> {
    pub fn login(&self, ident: &str, pwd: &str, totp: &str) -> Result<SessionToken, Box<dyn Error>> {
        info!("got a \"login\" request from email {} ", ident);        
        
        let user = if regex::match_regex(regex::EMAIL, ident).is_ok() {
            self.user_repo.find_by_email(ident)?
        } else {
            self.user_repo.find_by_name(ident)?
        };

        let shadowed_pwd = security::shadow(pwd, constants::PWD_SUFIX);
        if !user.match_password(&shadowed_pwd) {
            return Err(constants::ERR_NOT_FOUND.into());
        }

        // if, and only if, the user has activated the totp
        if let Ok(secret) = self.secret_repo.find_by_user_and_name(user.get_id(), constants::TOTP_SECRET_NAME) {
            if !secret.is_deleted() {
                if totp.len() == 0 {
                    return Err(constants::ERR_UNAUTHORIZED.into());
                }
    
                let data = secret.get_data();
                if !security::verify_totp(data, totp)? {
                    return Err(constants::ERR_UNAUTHORIZED.into());
                }
            }
        }

        if let Ok(token) = self.session_repo.find(user.get_id()) {
            return Ok(token);
        }

        let sess = SessionToken::new(constants::TOKEN_ISSUER, user.get_id(), Duration::from_secs(self.lifetime));
        self.session_repo.save(&sess)?;
        Ok(sess)
    }

    pub fn logout(&self, user_id: i32) -> Result<(), Box<dyn Error>> {
        info!("got a \"logout\" request from user id {} ", user_id);  
        self.session_repo.delete(user_id)
    }
}