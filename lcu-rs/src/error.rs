use reqwest::Error as ReqwestError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("no league client process is running")]
    NotRunning,
    #[error("error in request")]
    RequestError(#[from] ReqwestError),
}
