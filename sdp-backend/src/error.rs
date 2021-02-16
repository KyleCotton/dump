use actix_web::{dev::Body, http::StatusCode, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ApiError {
    CommandNotInTimeIssuedBuffer,
    DatabaseConnFailed,
    HashingFailed,
    LoginFailedUserNotExist,
    LoginFailedPasswordIncorrect,
    CmdInstructionNotSupported,
    RobotInitializationFailed,
    SerializationError,
    AuthenticationFailed,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_json = serde_json::to_string(&self).unwrap_or(String::new());
        write!(f, "{}", error_json)
    }
}

impl From<ApiError> for HttpResponse {
    fn from(error: ApiError) -> Self {
        // Get the json value for the error
        let error_json = match serde_json::to_string(&error) {
            Ok(j) => j,
            _ => return HttpResponse::InternalServerError().finish(),
        };

        // Give an appropiate error code for the error.
        match error {
            ApiError::CommandNotInTimeIssuedBuffer => HttpResponse::BadRequest().json(error_json),
            ApiError::DatabaseConnFailed => HttpResponse::InternalServerError().json(error_json),
            ApiError::HashingFailed => HttpResponse::InternalServerError().json(error_json),
            ApiError::LoginFailedUserNotExist => HttpResponse::Unauthorized().json(error_json),
            ApiError::LoginFailedPasswordIncorrect => HttpResponse::Unauthorized().json(error_json),
            ApiError::CmdInstructionNotSupported => HttpResponse::BadRequest().json(error_json),
            ApiError::RobotInitializationFailed => HttpResponse::BadRequest().json(error_json),
            ApiError::SerializationError => HttpResponse::InternalServerError().json(error_json),
            ApiError::AuthenticationFailed => HttpResponse::Unauthorized().json(error_json),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        let error: ApiError = self.clone();
        let res: HttpResponse = error.into();

        res.status()
    }

    fn error_response(&self) -> HttpResponse<Body> {
        let error: ApiError = self.clone();
        let res: HttpResponse = error.into();

        res.into_body()
    }
}
