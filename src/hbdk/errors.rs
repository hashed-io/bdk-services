use bitcoin;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response;
use rocket::response::{status, Responder, Response};
use rocket::serde::{json::Json, Deserialize, Serialize};
use std::error;
use std::fmt;
use xyzpub;

#[derive(Debug, Serialize)]
pub struct Error {
    details: String,
}

impl Error {
    pub fn new(msg: &str) -> Error {
        Error {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<xyzpub::Error> for Error {
    fn from(err: xyzpub::Error) -> Self {
        Error::new(&format!("{:#?}", err))
    }
}

impl From<bdk::Error> for Error {
    fn from(err: bdk::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl From<bdk::electrum_client::Error> for Error {
    fn from(err: bdk::electrum_client::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl From<bitcoin_hashes::hex::Error> for Error {
    fn from(err: bitcoin_hashes::hex::Error) -> Self {
        Error::new(&err.to_string())
    }
}


// impl From<Error> for Box<Error> {
//   fn from(err: xyzpub::Error) -> Self {
//       Error::new(format!("{:#?}", err))
//   }
// }

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'o> {
        status::BadRequest(Some(Json(self))).respond_to(req)
    }
}
