#[macro_use]
extern crate rocket;

use bdk_services::hbdk::{
    errors::Error, Blockchain, Descriptors, Multisig, SignedTrx, Trx, TrxDetails, Wallet,
};
use bitcoin::Network;
use rocket::serde::{json::Json, Deserialize};
use rocket::State;
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{AdHoc, Fairing, Info, Kind};
use std::path::PathBuf;

#[derive(Deserialize)]
struct Config {
    network_url: String,
    network: Network,
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[options("/<path..>")]
fn options(path: PathBuf){}


/// Returns a new address for the provided output descriptor
///
/// # Arguments
///
/// * `descriptors` - A Descriptors object with the descriptor field set, the change descriptor is optional
///
/// # Errors
/// 
/// Returns 404 error in case of an invalid descriptor
#[post("/gen_new_address", data = "<descriptors>")]
fn gen_new_address(
    config: &State<Config>,
    descriptors: Json<Descriptors>,
) -> Result<String, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &descriptors)?;
    let address = wallet.get_new_address()?;
    Ok(address.to_string())
}

/// Returns a list of trxs for the provided output descriptors
///
/// # Arguments
///
/// * `descriptors` - A Descriptors object with the descriptor field set, the change descriptor is optional
///
/// # Errors
/// 
/// Returns 404 error in case of an invalid descriptor
#[post("/list_trxs", data = "<descriptors>")]
fn list_trxs(
    config: &State<Config>,
    descriptors: Json<Descriptors>,
) -> Result<Json<Vec<TrxDetails>>, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &descriptors)?;
    let trxs = wallet.list_trxs()?;
    Ok(Json(trxs))
}

/// Returns a Multisig object for the provided output descriptor
///
/// # Arguments
///
/// * `descriptors` - A Descriptors object with the descriptor field set, the change descriptor is optional
///
/// # Errors
/// 
/// Returns 404 error in case of an invalid descriptor
#[post("/get_multisig", data = "<descriptors>")]
fn gen_multisig(
    config: &State<Config>,
    descriptors: Json<Descriptors>,
) -> Result<Json<Multisig>, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &descriptors)?;
    Ok(Json(wallet.get_multisig()?))
}

/// Returns a Descriptor object with the descriptor and change_descriptor fields set for the provided multisig
///
/// # Arguments
///
/// * `multisig` - A Multisig object, the cosigner xpub details can be provided in the separate fields or the
/// full xpub can be provided in the xpub field, and it will be parsed to obtain the details
///
/// # Errors
/// 
/// Returns 404 error in case of an invalid multisig
#[post("/gen_output_descriptor", data = "<multisig>")]
fn gen_output_descriptor(
    config: &State<Config>,
    multisig: Json<Multisig>,
) -> Result<Json<Descriptors>, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_multisig(&blockchain, &multisig)?;
    let descriptors = wallet.get_descriptors()?;
    Ok(Json(descriptors))
}

/// Returns a psbt as a base64 encoded string for the provided Trx object
///
/// # Arguments
///
/// * `trx` - A Trx object with the output descriptor and trx details to use
///
/// # Errors
/// 
/// Returns 404 error in case of an invalid trx
#[post("/gen_psbt", data = "<trx>")]
fn gen_psbt(config: &State<Config>, trx: Json<Trx>) -> Result<String, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &trx.descriptors)?;
    Ok(wallet.build_tx_encoded(&trx)?)
}

/// Finalizes and broadcasts a trx bsaed on the provided signed psbts, returns the trx ID in
/// case of success
///
/// # Arguments
///
/// * `signed_trx` - A SignedTrx object with the output descriptor and signed psbts
///
/// # Errors
/// 
/// Returns 404 error in case of an invalid signed trx object
#[post("/finalize_trx", data = "<signed_trx>")]
fn finalize_trx(config: &State<Config>, signed_trx: Json<SignedTrx>) -> Result<String, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &signed_trx.descriptors)?;
    Ok(wallet.finalize_trx(signed_trx.psbts.as_slice())?)
}

/// Returns balance in sats for the provided output descriptor
///
/// # Arguments
///
/// * `descriptors` - A Descriptors object with the descriptor field set, the change descriptor is optional
///
/// # Errors
/// 
/// Returns 404 error in case of an invalid descriptor
#[post("/get_balance", data = "<descriptors>")]
fn get_balance(
    config: &State<Config>,
    descriptors: Json<Descriptors>,
) -> Result<String, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &descriptors)?;
    let balance = wallet.get_balance()?;
    Ok(balance.to_string())
}

#[get("/")]
fn index() -> &'static str {
    "Root service not implemented"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                index,
                gen_output_descriptor,
                gen_new_address,
                gen_psbt,
                finalize_trx,
                gen_multisig,
                get_balance,
                list_trxs,
                options
            ],
        )
        .attach(AdHoc::config::<Config>())
        .attach(CORS)
}
