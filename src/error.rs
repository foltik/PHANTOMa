/*
use std::error::Error as StdError;
use failure::{Fail, Error, Backtrace};
use std::fmt::{self, Display, Formatter};

use rendy::hal;

#[derive(Fail, Debug)]
pub enum AppError {
    #[fail(display = "A hal error occurred: {}.", _0)]
    HalError(Box<dyn std::error::Error>)
}

impl<T: Into<Error>> From<T> for AppError {
    fn from(t: T) -> AppError { AppError(t.into()) }
}
*/