#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

// use bdk::bitcoin;
// use bdk::Wallet;
// use bdk::database::MemoryDatabase;
// use bdk::wallet::AddressIndex::New;
// use bitcoin::Network;

// #[derive(Deserialize)]
// struct MultisigWallet>'r> {
//     threshold: u32,
//     xpubs: u32,
// }

use serde::Deserialize;
use rocket_contrib::json::Json;

// #[derive(FromForm)]
// struct Task {
//     complete: String,
//     description: String,
// }

// #[post("/todo", data = "<task>")]
// fn todo(task: Form<Task>) { 
//     println!("{:?}", task.description);
//     println!("{:?}", task.complete);
// }


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


// #[get("/gen_output_descriptor")]
// fn gen_output_descriptor(_threshold: u32, _xpubs: [String; 15]) -> String {

//     // input is a list of xpubs and the multisig threshold
//     // output is the properly formatted output descriptor
//     return String::from("")
// }

// #[get("/get_balance")]
// fn get_balance(descriptor: &str) -> u32 {

//     // get balance by querying the bitcoin node
//     // return balance in sats

//     return 0
// }

// #[get("/")]
// fn index() -> &'static str {
//     "Root service not implemented"
// }

// #![feature(proc_macro_hygiene, decl_macro)]

// #[macro_use] use rocket::*;


#[get("/echo/<echo>")]
fn echo_fn(echo: String) -> String {
    format!("{}", echo)
}

#[derive(Deserialize)]
struct MyParam {
    value: String
}

#[post("/echo-post/<echo>")]
fn echo_post(echo: String) -> String {
    format!("{}", echo)
}

#[post("/submit", format= "application/json", data = "<user_input>")]
fn submit_task(user_input: Json<MyParam>) -> String {
    format!("Your value: {:?}", user_input.value)
}

fn main() {
    rocket::ignite().mount("/", routes![echo_fn, echo_post, submit_task]).launch();
}


// fn main() {
//     rocket::ignite().mount("/", routes![new]).launch();
// }
