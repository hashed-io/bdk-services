#[macro_use]
extern crate rocket;

use bdk::bitcoin;
use bdk::Wallet;
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex::New;
use bitcoin::Network;

#[get("/gen_new_address")]
fn gen_new_address() -> String {
    let descriptor = "wpkh(tpubD6NzVbkrYhZ4Xferm7Pz4VnjdcDPFyjVu5K4iZXQ4pVN8Cks4pHVowTBXBKRhX64pkRyJZJN5xAKj4UDNnLPb5p2sSKXhewoYx5GbTdUFWq/*)";
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
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, gen_new_address])
}
