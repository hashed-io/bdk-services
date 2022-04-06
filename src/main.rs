#[macro_use]
extern crate rocket;

use bdk::bitcoin;
use bdk::{Wallet, SyncOptions};
use bdk::blockchain::ElectrumBlockchain;
use bdk::electrum_client::Client;
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex::New;
use bitcoin::Network;

 // http://localhost:8000/gen_new_address/wpkh%28tpubDDYkZojQFQjht8Tm4jsS3iuEmKjTiEGjG6KnuFNKKJb5A6ZUCUZKdvLdSDWofKi4ToRCwb9poe1XdqfUnP4jaJjCB2Zwv11ZLgSbnZSNecE%2F%2A%29
#[get("/gen_new_address/<descriptor>")]
fn gen_new_address(descriptor: &str) -> String {

    // create a new wallet from the descriptor string
    let wallet = Wallet::new(
        descriptor,
        // TODO: parameterize change_descriptor
        None,
        // TODO: parameterize testnet vs mainnet vs regtest
        Network::Testnet,
        MemoryDatabase::default(),
    ).unwrap();

    // print!("{}", descriptor);

    // TODO: this is always the "first" address; to get the "next" address,
    // we'll need to check to loop through addresses to check for an existing UTXO (bitcoin balance).
    // In order to check the balance, we'll need to connect to a bitcoin node.
    // The paramter to `get_address` is an `AddressIndex` - maybe this allows for a "next" option.
    let address = wallet.get_address(New).unwrap();

    // this is just to test the iteration of new addresses
    // println!("Address #0: {}", address.to_string());
    // println!("Address #1: {}", wallet.get_address(New).unwrap().to_string());
    // println!("Address #2: {}", wallet.get_address(New).unwrap().to_string());
    // return the address as an http request
    address.to_string().into()
}

// #[get("/gen_output_descriptor")]
// fn gen_output_descriptor(_threshold: u32, _xpubs: [String; 15]) -> String {

//     // input is a list of xpubs and the multisig threshold
//     // output is the properly formatted output descriptor
//     return String::from("")
// }

#[get("/get_balance/<descriptor>")]
fn get_balance(descriptor: &str) -> String {


    let blockchain = ElectrumBlockchain::from(
        Client::new("ssl://electrum.blockstream.info:60002").unwrap()
    );

    // create a new wallet from the descriptor string
    let wallet = Wallet::new(
        descriptor,
        // TODO: parameterize change_descriptor
        None,
        // TODO: parameterize testnet vs mainnet vs regtest
        Network::Testnet,
        MemoryDatabase::default(),
    ).unwrap();
    // get balance by querying the bitcoin node
    // return balance in sats

    wallet.sync(&blockchain, SyncOptions::default()).unwrap();

    return format!("Descriptor balance: {} SAT", wallet.get_balance().unwrap());
}


#[get("/")]
fn index() -> &'static str {
    "Root service not implemented"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, gen_new_address, get_balance])
}