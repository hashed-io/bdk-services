#[macro_use]
extern crate rocket;

use core::str::FromStr;
use bdk::bitcoin;
// use bdk::Wallet;
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex::New;
use bitcoin::Network;
use bdk_services::hbdk::{Multisig, Descriptors, Wallet, Blockchain, errors::Error};
use rocket::serde::{Deserialize, json::Json};
use rocket::State;
use rocket::fairing::AdHoc;

#[derive(Deserialize)]
struct Config {
    network_url: String,
    network: Network,
}
// #[get("/gen_new_address/<descriptor>")]
// fn gen_new_address(descriptor: &str) -> String {

//     // create a new wallet from the descriptor string
//     let wallet = Wallet::new(
//         descriptor,
//         // TODO: parameterize change_descriptor
//         None,
//         // TODO: parameterize testnet vs mainnet vs regtest
//         Network::Testnet,
//         MemoryDatabase::default(),
//     ).unwrap();

//     // TODO: this is always the "first" address; to get the "next" address,
//     // we'll need to check to loop through addresses to check for an existing UTXO (bitcoin balance).
//     // In order to check the balance, we'll need to connect to a bitcoin node.
//     // The paramter to `get_address` is an `AddressIndex` - maybe this allows for a "next" option.
//     let address = wallet.get_address(New).unwrap();

//     // return the address as an http request
//     address.to_string().into()
// }

#[post("/gen_output_descriptor", data = "<multisig>")]
fn gen_output_descriptor(config: &State<Config>, multisig: Json<Multisig>) -> Result<Json<Descriptors>, Error> {

    // input is a list of xpubs and the multisig threshold
    // output is the properly formatted output descriptor
    let blockchain = Blockchain::new(&config.network_url, config.network).unwrap();
    let wallet = Wallet::from_multisig(blockchain, &multisig)?;
    let descriptors = wallet.get_descriptors()?;
    Ok(Json(descriptors))

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
    .mount("/", routes![index, gen_output_descriptor])
    .attach(AdHoc::config::<Config>())
}
