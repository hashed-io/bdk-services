#[macro_use]
extern crate rocket;

use bdk_services::hbdk::{
    errors::Error, Blockchain, Cosigner, Descriptors, Multisig, SignedTrx, Trx, TrxDetails,
    VerifyPSBTPayload, Wallet,
};
use bdk_services::hbdk::{ProofOfReserves, ProofOfReservesRequest, SignedProofOfReserves};
use bitcoin::Network;
use rocket::fairing::{AdHoc, Fairing, Info, Kind};
use rocket::http::Header;
use rocket::serde::{json::Json, Deserialize};
use rocket::State;
use rocket::{Request, Response};
use std::path::PathBuf;

#[derive(Deserialize)]
struct Config {
    network_url: String,
    network: Network,
    pub_key_search_radius: u8,
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[options("/<path..>")]
fn options(path: PathBuf) {}

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

/// Returns a list of xpubs who signed the psbt
///
/// # Arguments
///
/// * `verify_psbt_payload` - A VerifyPSBTPayload object with the descriptors and psbt fields set
///
/// # Errors
///
/// Returns 404 error in case of an invalid descriptors or psbt
#[post("/list_signers", data = "<verify_psbt_payload>")]
fn list_signers(
    config: &State<Config>,
    verify_psbt_payload: Json<VerifyPSBTPayload>,
) -> Result<Json<Vec<Cosigner>>, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &verify_psbt_payload.descriptors)?;
    let signers = wallet.get_signers(&verify_psbt_payload.psbt, config.pub_key_search_radius)?;
    Ok(Json(signers))
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

/// Returns proof of reserves as a base64 encoded psbt string for the wallet described by the descriptors
///
/// # Arguments
///
/// * `proof_of_reserves_req` - A ProofOfReserves object with the output descriptor and message to use to generate the proof
///
/// # Errors
///
/// Returns 404 error in case of an invalid descriptors
#[post("/create_proof", data = "<proof_of_reserves_req>")]
fn create_proof_of_reserves(
    config: &State<Config>,
    proof_of_reserves_req: Json<ProofOfReservesRequest>,
) -> Result<String, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &proof_of_reserves_req.descriptors)?;
    Ok(wallet.create_proof_of_reserves_encoded(&proof_of_reserves_req.message)?)
}

/// Finalizes and broadcasts a trx based on the provided signed psbts, returns the trx ID in
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
    Ok(wallet.finalize_trx(signed_trx.psbts.as_slice(), signed_trx.broadcast)?)
}

/// Finalizes proof based on the provided signed psbts, returns the combined psbt in
/// case of success
///
/// # Arguments
///
/// * `signed_proof` - A SignedProofOfReserves object with the output descriptor and signed psbts
///
/// # Errors
///
/// Returns 404 error in case of an invalid signed proof of reserves object
#[post("/finalize_proof", data = "<signed_proof>")]
fn finalize_proof_of_reserves(
    config: &State<Config>,
    signed_proof: Json<SignedProofOfReserves>,
) -> Result<String, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &signed_proof.descriptors)?;
    Ok(wallet.finalize_proof_of_reserves(signed_proof.psbts.as_slice())?)
}

/// Verify proof of reserves
///
///
/// # Arguments
///
/// * `proof_of_reserves` - A ProofOfReverse object with the output descriptor, finalized psbt and message
///
/// # Errors
///
/// Returns 404 error in case of an invalid signed proof of reserves object
#[post("/verify_proof", data = "<proof_of_reserves>")]
fn verify_proof_of_reserves(
    config: &State<Config>,
    proof_of_reserves: Json<ProofOfReserves>,
) -> Result<String, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &proof_of_reserves.descriptors)?;
    Ok(wallet
        .verify_proof_of_reserves(&proof_of_reserves.message, &proof_of_reserves.psbt)?
        .to_string())
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
fn get_balance(config: &State<Config>, descriptors: Json<Descriptors>) -> Result<String, Error> {
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
                list_signers,
                create_proof_of_reserves,
                finalize_proof_of_reserves,
                verify_proof_of_reserves,
                options
            ],
        )
        .attach(AdHoc::config::<Config>())
        .attach(CORS)
}
