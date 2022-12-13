use bitcoin::consensus::encode;
use bitcoin::locktime;
use bitcoin::util::address;
use bitcoin::util::psbt;
use rocket::request::Request;
use rocket::response;
use rocket::response::{status, Responder};
use rocket::serde::{json::Json, Serialize};
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

impl From<miniscript::descriptor::ConversionError> for Error {
    fn from(err: miniscript::descriptor::ConversionError) -> Self {
        Error::new(&err.to_string())
    }
}

impl From<miniscript::descriptor::DescriptorKeyParseError> for Error {
    fn from(err: miniscript::descriptor::DescriptorKeyParseError) -> Self {
        Error::new(&err.to_string())
    }
}

impl From<miniscript::interpreter::Error> for Error {
    fn from(err: miniscript::interpreter::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl From<bitcoin::util::bip32::Error> for Error {
    fn from(err: bitcoin::util::bip32::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl From<bitcoin_hashes::hex::Error> for Error {
    fn from(err: bitcoin_hashes::hex::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl From<encode::Error> for Error {
    fn from(err: encode::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl From<psbt::Error> for Error {
    fn from(err: psbt::Error) -> Self {
        Error::new(&err.to_string())
    }
}
impl From<address::Error> for Error {
    fn from(err: address::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl From<locktime::Error> for Error {
    fn from(err: locktime::Error) -> Self {
        Error::new(&err.to_string())
    }
}

impl From<bdk::miniscript::descriptor::ConversionError> for Error {
    fn from(err: bdk::miniscript::descriptor::ConversionError) -> Self {
        Error::new(&err.to_string())
    }
}

impl From<bdk::miniscript::descriptor::DescriptorKeyParseError> for Error {
    fn from(err: bdk::miniscript::descriptor::DescriptorKeyParseError) -> Self {
        Error::new(&err.to_string())
    }
}

impl From<bdk_reserves::reserves::ProofError> for Error {
    fn from(err: bdk_reserves::reserves::ProofError) -> Self {
        Error::new(&format!("ProofError: {:?}", err))
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
