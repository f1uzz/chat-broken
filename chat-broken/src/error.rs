use lcu::ApiError;
use reqwest::Error as ReqwestError;
use std::io::Error as IoError;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChatBrokenError {
    #[error("lcu api error")]
    ApiError(#[from] ApiError),
    #[error("reqwest error")]
    ReqwestError(#[from] ReqwestError),
    #[error("lcu api returned invalid data")]
    InvalidDataError(&'static str),
    #[error("error with terminal input")]
    IoError(#[from] IoError),
    #[error("could not parse string as int")]
    ParseIntError(#[from] ParseIntError),
    #[error("invalid index")]
    InvalidIndexError,
}
