#[macro_use]
extern crate rocket;

use bdk::bitcoin;
use bdk::Wallet;
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex::New;
use bitcoin::Network;

#[get("/gen_new_address/<descriptor>")]
fn gen_new_address(descriptor: &str) -> String {
    let wallet = Wallet::new(
        descriptor,
        None,
        Network::Testnet,
        MemoryDatabase::default(),
    ).unwrap();
    // wallet.add_address_validator(Arc::new(PrintAddressAndContinue));
    let address = wallet.get_address(New).unwrap();
    address.to_string().into()
}

#[get("/")]
fn index() -> &'static str {
    "Root service not implemented"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, gen_new_address])
}