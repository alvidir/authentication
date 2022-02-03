use tonic::{Request, Response, Status};
use crate::security;
use crate::constants;
use crate::user::application::{UserRepository, UserApplication};
use crate::session::application::SessionRepository;
use crate::secret::application::SecretRepository;
use crate::smtp::Mailer;
use crate::grpc;

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("user");
}

// Proto generated server traits
use proto::user_server::User;
pub use proto::user_server::UserServer;

// Proto message structs
use proto::{SignupRequest, ResetPasswordRequest, DeleteRequest, TotpRequest, Empty};

pub struct UserImplementation<
    U: UserRepository + Sync + Send,
    E:  SecretRepository + Sync + Send,
    S: SessionRepository + Sync + Send,
    M: Mailer,
    > {
    pub user_app: UserApplication<U, E, S, M>,
    pub rsa_secret: &'static [u8],
    pub rsa_public: &'static [u8],
    pub jwt_secret: &'static [u8],
    pub jwt_public: &'static [u8],
    pub jwt_header: &'static str,
    pub allow_unverified: bool,
}

#[tonic::async_trait]
impl<
    U: 'static + UserRepository + Sync + Send,
    E: 'static + SecretRepository + Sync + Send,
    S: 'static + SessionRepository + Sync + Send,
    M: 'static + Mailer + Sync + Send,
    > User for UserImplementation<U, E, S, M> {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<Empty>, Status> {
        if request.metadata().get(self.jwt_header).is_none() {
            let msg_ref = request.into_inner();
            let shadowed_pwd = security::shadow(&msg_ref.pwd, constants::PWD_SUFIX);

            if !self.allow_unverified {
                self.user_app.verify_user(&msg_ref.email, &shadowed_pwd, self.jwt_secret)
                    .map_err(|err| {
                        error!("{}: {}", constants::ERR_SEND_EMAIL, err);
                        Status::aborted(constants::ERR_SEND_EMAIL)
                    })?;
                
                return Err(Status::failed_precondition(constants::ERR_UNVERIFIED))
            }

            match self.user_app.signup(&msg_ref.email, &shadowed_pwd) {
                Err(err) => return Err(Status::aborted(err.to_string())),
                Ok(_) => return Ok(Response::new(Empty{})),
            };
        }
        
        let token = grpc::get_header(&request, self.jwt_header)?;
        let token = base64::decode(token)
            .map_err(|err| {
                warn!("{}: {}", constants::ERR_PARSE_HEADER, err);
                Status::aborted(constants::ERR_PARSE_HEADER)
            })?;

        let token = security::decrypt(self.rsa_secret, &token)
            .map_err(|err| {
                warn!("{}: {}", constants::ERR_DECRYPT_TOKEN, err);
                Status::aborted(constants::ERR_DECRYPT_TOKEN)
            })?;

        match self.user_app.secure_signup(&base64::encode(token), self.jwt_public) {
            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(_) => Ok(Response::new(Empty{})),
        }
    }

    async fn reset_password(&self, _: Request<ResetPasswordRequest>) -> Result<Response<Empty>, Status> {
        return Err(Status::unimplemented("not implemented".to_string()));
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_header(&request, self.jwt_header)?;
        let msg_ref = request.into_inner();
        
        let shadowed_pwd = security::shadow(&msg_ref.pwd, constants::PWD_SUFIX);
        match self.user_app.secure_delete(&shadowed_pwd, &msg_ref.totp, &token, self.jwt_public) {
            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(()) => Ok(Response::new(Empty{})),
        }
    }

    async fn totp(&self, request: Request<TotpRequest>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_header(&request, self.jwt_header)?;
        let msg_ref = request.into_inner();
        let shadowed_pwd = security::shadow(&msg_ref.pwd, constants::PWD_SUFIX);

        if msg_ref.action == 0 {
            match self.user_app.secure_enable_totp(&shadowed_pwd, &msg_ref.totp, &token, self.jwt_public) {
                Err(err) => return Err(Status::unknown(err.to_string())),
                Ok(token) => {
                    let mut response = Response::new(Empty{});
                    response.metadata_mut().insert(self.jwt_header, token.parse().unwrap());
                    return Ok(response);
                }
            }
        }

        if msg_ref.action == 1 {
            match self.user_app.secure_disable_totp(&shadowed_pwd, &msg_ref.totp, &token, self.jwt_public) {
                Ok(_) => return Ok(Response::new(Empty{})),
                Err(err) => return Err(Status::unknown(err.to_string())),
            }
        }

        Err(Status::invalid_argument(constants::ERR_INVALID_OPTION))
    }
}