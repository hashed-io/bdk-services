#[macro_use]
extern crate rocket;

use bdk::bitcoin;
use bdk_services::hbdk::{
    errors::Error, Blockchain, Descriptors, Multisig, SignedTrx, Trx, Wallet,
};
use bitcoin::Network;
use rocket::fairing::AdHoc;
use rocket::serde::{json::Json, Deserialize};
use rocket::State;

#[derive(Deserialize)]
struct Config {
    network_url: String,
    network: Network,
}

#[post("/gen_new_address", data = "<descriptors>")]
fn gen_new_address(
    config: &State<Config>,
    descriptors: Json<Descriptors>,
) -> Result<String, Error> {
    // input is a list of xpubs and the multisig threshold
    // output is the properly formatted output descriptor
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &descriptors)?;
    let address = wallet.get_new_address()?;
    Ok(address.to_string())
}

#[post("/get_multisig", data = "<descriptors>")]
fn gen_multisig(
    config: &State<Config>,
    descriptors: Json<Descriptors>,
) -> Result<Json<Multisig>, Error> {
    // input is a list of xpubs and the multisig threshold
    // output is the properly formatted output descriptor
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &descriptors)?;
    Ok(Json(wallet.get_multisig()?))
}

#[post("/gen_output_descriptor", data = "<multisig>")]
fn gen_output_descriptor(
    config: &State<Config>,
    multisig: Json<Multisig>,
) -> Result<Json<Descriptors>, Error> {
    // input is a list of xpubs and the multisig threshold
    // output is the properly formatted output descriptor
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_multisig(&blockchain, &multisig)?;
    let descriptors = wallet.get_descriptors()?;
    Ok(Json(descriptors))
}

#[post("/gen_psbt", data = "<trx>")]
fn gen_psbt(config: &State<Config>, trx: Json<Trx>) -> Result<String, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &trx.descriptors)?;
    Ok(wallet.build_tx_encoded(&trx)?)
}

#[post("/finalize_trx", data = "<signed_trx>")]
fn finalize_trx(config: &State<Config>, signed_trx: Json<SignedTrx>) -> Result<String, Error> {
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &signed_trx.descriptors)?;
    Ok(wallet.finalize_trx(signed_trx.psbts.as_slice())?)
}

// #[get("/get_balance")]
// fn get_balance(descriptor: &str) -> u32 {

//     // get balance by querying the bitcoin node
//     // return balance in sats

//     return 0
// }

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
            ],
        )
        .attach(AdHoc::config::<Config>())
}
