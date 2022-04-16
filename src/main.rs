#[macro_use]
extern crate rocket;

use core::str::FromStr;
use bdk::bitcoin;
// use bdk::Wallet;
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex::New;
use bitcoin::Network;
use bdk_services::hbdk::{Multisig, Descriptors, Wallet, Blockchain, Trx, errors::Error};
use rocket::serde::{Deserialize, json::Json};
use rocket::State;
use rocket::fairing::AdHoc;

#[derive(Deserialize)]
struct Config {
    network_url: String,
    network: Network,
}

#[post("/gen_new_address", data = "<descriptors>")]
fn gen_new_address(config: &State<Config>, descriptors: Json<Descriptors>) -> Result<String, Error> {

    // input is a list of xpubs and the multisig threshold
    // output is the properly formatted output descriptor
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &descriptors)?;
    let address = wallet.get_new_address()?;
    Ok(address.to_string())

}

#[post("/gen_output_descriptor", data = "<multisig>")]
fn gen_output_descriptor(config: &State<Config>, multisig: Json<Multisig>) -> Result<Json<Descriptors>, Error> {

    // input is a list of xpubs and the multisig threshold
    // output is the properly formatted output descriptor
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_multisig(&blockchain, &multisig)?;
    let descriptors = wallet.get_descriptors()?;
    Ok(Json(descriptors))

}

#[post("/gen_psbt", data = "<trx>")]
fn gen_psbt(config: &State<Config>, trx: Json<Trx>) -> Result<String, Error> {

    // input is a list of xpubs and the multisig threshold
    // output is the properly formatted output descriptor
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_descriptors(&blockchain, &trx.descriptors)?;
    Ok(wallet.build_tx_encoded(&trx)?)

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
    .mount("/", routes![index, gen_output_descriptor, gen_new_address, gen_psbt])
    .attach(AdHoc::config::<Config>())
}
