pub mod errors;
pub mod util;

use bdk::blockchain::{Blockchain as BlockchainTrait, ElectrumBlockchain};
use bdk::database::MemoryDatabase;
use bdk::electrum_client::Client;
use bdk::wallet::export::WalletExport;
use bdk::wallet::{AddressIndex, AddressInfo};
use bdk::{FeeRate, KeychainKind, SignOptions, SyncOptions, TransactionDetails};
use bitcoin::blockdata::script::Script;
use bitcoin::consensus;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::util::address::{Address, AddressType};
use bitcoin::{Network, Transaction};
use core::str::FromStr;
use errors::Error;
use rocket::serde::{json::Json, Deserialize, Serialize};
use std::clone::Clone;
use std::error;

#[derive(Deserialize, Serialize)]
pub struct Cosigner {
  pub xfp: Option<String>,
  pub xpub: String,
  pub derivation_path: Option<String>,
}

impl Cosigner {
  pub fn descriptor(&self, change: bool) -> Result<String, Error> {
    let mut s = String::new();
    let child = if change { "/1/*" } else { "/0/*" };
    if self.xfp.is_some() && self.derivation_path.is_some() {
      let xfp = self.xfp.as_ref().unwrap();
      let path = self.derivation_path.as_ref().unwrap();
      let path = path.trim_start_matches("m/").trim_start_matches("/");
      s = format!("[{}/{}]", xfp, path);
    }
    if !util::is_multisig_xpub(&self.xpub) {
      return Err(
        bdk::Error::Generic(format!("xpub is not a multisig xpub. xpub:{}", self.xpub)).into(),
      );
    }
    Ok(format!(
      "{}{}{}",
      s,
      util::to_legacy_xpub(&self.xpub)?,
      child
    ))
  }
}

#[derive(Deserialize, Serialize)]
pub struct Multisig {
  pub threshold: u32,
  pub cosigners: Vec<Cosigner>,
}

impl Multisig {
  pub fn new(threshold: u32) -> Self {
    return Multisig {
      threshold,
      cosigners: Vec::new(),
    };
  }

  pub fn add_cosigner(&mut self, cosigner: Cosigner) {
    self.cosigners.push(cosigner);
  }

  pub fn descriptor(&self, change: bool) -> Result<String, Error> {
    if self.threshold > self.cosigners.len() as u32 {
      return Err(
        errors::Error::new(&format!(
          "multisig threshold: {} is greater than the number of cosigners: {}",
          self.threshold,
          self.cosigners.len()
        ))
        .into(),
      );
    }
    let mut descriptor = String::new();
    for cosigner in &self.cosigners {
      descriptor = format!("{},{}", descriptor, cosigner.descriptor(change)?);
    }
    Ok(format!(
      "wsh(sortedmulti({}{}))",
      self.threshold, descriptor
    ))
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Descriptors {
  pub descriptor: String,
  pub change_descriptor: Option<String>,
}

impl Descriptors {
  pub fn new(descriptor: String, change_descriptor: String) -> Descriptors {
    Descriptors {
      descriptor,
      change_descriptor: Some(change_descriptor),
    }
  }

  pub fn from_descriptor(descriptor: String) -> Descriptors {
    Descriptors {
      descriptor,
      change_descriptor: None,
    }
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Trx {
  pub descriptors: Descriptors,
  pub to_address: String,
  pub amount: u64,
  pub fee_sat_per_vb: f32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SignedTrx {
  pub descriptors: Descriptors,
  pub psbts: Vec<String>,
}


pub struct Blockchain {
  blockchain: ElectrumBlockchain,
  network: Network,
}

impl Blockchain {
  pub fn new(url: &str, network: Network) -> Result<Self, Error> {
    let client = Client::new(url)?;
    return Ok(Blockchain {
      blockchain: ElectrumBlockchain::from(client),
      network,
    });
  }

  pub fn get_blockchain(&self) -> &ElectrumBlockchain {
    &self.blockchain
  }

  pub fn broadcast(&self, tx: &Transaction)-> Result<(), Error> {
    self.blockchain.broadcast(tx)?;
    Ok(())
  }
}

pub struct Wallet<'a> {
  blockchain: &'a Blockchain,
  wallet: bdk::Wallet<MemoryDatabase>,
}

impl<'a> Wallet<'a> {
  pub fn from_multisig(blockchain: &'a Blockchain, multisig: &Multisig) -> Result<Self, Error> {
    Self::from_descriptors(
      blockchain,
      &Descriptors::new(multisig.descriptor(false)?, multisig.descriptor(true)?),
    )
  }

  pub fn from_descriptors(
    blockchain: &'a Blockchain,
    descriptors: &Descriptors,
  ) -> Result<Self, Error> {
    let network = blockchain.network.clone();
    Ok(Wallet {
      blockchain,
      wallet: bdk::Wallet::new(
        &descriptors.descriptor,
        descriptors.change_descriptor.as_ref(),
        network,
        MemoryDatabase::default(),
      )?,
    })
  }

  pub fn get_descriptors(&self) -> Result<Descriptors, Error> {
    let descriptor = self
      .wallet
      .public_descriptor(KeychainKind::External)?
      .ok_or(bdk::Error::Generic("No descriptor for wallet".to_string()))?;
    let change_descriptor = self.wallet.public_descriptor(KeychainKind::Internal)?;
    let change_descriptor = change_descriptor.map(|desc| desc.to_string());
    return Ok(Descriptors {
      descriptor: descriptor.to_string(),
      change_descriptor,
    });
  }

  pub fn get_new_address(&self) -> Result<AddressInfo, Error> {
    self.sync()?;
    let address = self.wallet.get_address(AddressIndex::LastUnused)?;
    Ok(address)
  }

  pub fn get_balance(&self) -> Result<u64, Error> {
    self.sync()?;
    let balance = self.wallet.get_balance()?;
    Ok(balance)
  }

  pub fn build_tx(
    &self,
    trx: &Trx,
  ) -> Result<(PartiallySignedTransaction, TransactionDetails), Error> {
    self.sync()?;
    let mut builder = self.wallet.build_tx();
    // let to_wallet = Wallet::from_descriptors(self.blockchain, &Descriptors::from_descriptor(trx.to_pub_key.clone()))?;
    // to_wallet.sync()?;
    builder
      // .add_recipient(to_wallet.get_new_address()?.script_pubkey(), trx.amount)
      .add_recipient(
        Address::from_str(&trx.to_address)?.script_pubkey(),
        trx.amount,
      )
      .enable_rbf()
      .do_not_spend_change()
      .fee_rate(FeeRate::from_sat_per_vb(trx.fee_sat_per_vb));
    Ok(builder.finish()?)
  }

  pub fn build_tx_encoded(&self, trx: &Trx) -> Result<String, Error> {
    let (psbt, _) = self.build_tx(trx)?;
    Ok(base64::encode(consensus::serialize(&psbt)))
  }

  pub fn finalize_trx(&self, psbts: &[String])-> Result<String, Error> {
    if psbts.len() < 2 {
      return Err(Error::new(&format!("failed to finalized tx, there are less than required psbts, found: {}", psbts.len())));
    }
    let mut combined = self.deserialize_psbt(&psbts[0])?;
    for psbt in &psbts[1..] {
      combined.merge(self.deserialize_psbt(psbt)?)?;
    }
    self.sync()?;
    let finalized = self.wallet.finalize_psbt(&mut combined, SignOptions::default())?;
    if !finalized {
      return Err(Error::new("provided psbts do not finalize tx"));
    }
    let tx = combined.extract_tx();
    self.blockchain.broadcast(&tx)?;
    Ok(tx.txid().to_string())
  }

  fn deserialize_psbt(&self, psbt: &str)-> Result<PartiallySignedTransaction, Error> {
    let decoded = base64::decode(psbt).unwrap_or(psbt.as_bytes().to_vec());
    let deserialized = consensus::deserialize(&decoded)?;
    Ok(deserialized)
  }

  fn sync(&self) -> Result<(), Error> {
    self
      .wallet
      .sync(self.blockchain.get_blockchain(), SyncOptions::default())?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {

  use crate::hbdk::*;
  #[test]
  fn test_cosigner_descriptor() {
    let cosigner = Cosigner{
          xfp:Some("20F24288".to_string()),
          derivation_path: Some("m/48'/0'/0'/2".to_string()),
          xpub: "Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi".to_string(),
        };

    assert_eq!(cosigner.descriptor(false).unwrap(), "[20F24288/48'/0'/0'/2]xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/0/*");

    let cosigner = Cosigner{
      xfp:Some("20F24288".to_string()),
      derivation_path: Some("m/48'/0'/0'/2".to_string()),
      xpub: "Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi".to_string(),
    };

    assert_eq!(cosigner.descriptor(true).unwrap(), "[20F24288/48'/0'/0'/2]xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/1/*");

    let cosigner = Cosigner{
          xfp:Some("20F24288".to_string()),
          derivation_path: Some("/48'/0'/0'/2".to_string()),
          xpub: "Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi".to_string(),
        };

    assert_eq!(cosigner.descriptor(false).unwrap(), "[20F24288/48'/0'/0'/2]xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/0/*");

    let cosigner = Cosigner{
          xfp:Some("20F24288".to_string()),
          derivation_path: Some("48'/0'/0'/2".to_string()),
          xpub: "Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi".to_string(),
        };

    assert_eq!(cosigner.descriptor(false).unwrap(), "[20F24288/48'/0'/0'/2]xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/0/*");

    let cosigner = Cosigner{
          xfp:None,
          derivation_path: Some("m/48'/0'/0'/2".to_string()),
          xpub: "Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi".to_string(),
        };

    assert_eq!(cosigner.descriptor(false).unwrap(), "xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/0/*");

    let cosigner = Cosigner{
      xfp:Some("20F24288".to_string()),
      derivation_path: None,
      xpub: "Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi".to_string(),
    };

    assert_eq!(cosigner.descriptor(false).unwrap(), "xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/0/*");

    let cosigner = Cosigner{
      xfp:None,
      derivation_path: None,
      xpub: "Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi".to_string(),
    };

    assert_eq!(cosigner.descriptor(false).unwrap(), "xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/0/*");
  }

  #[test]
  fn test_cosigner_descriptor_should_fail_for_non_multisig_xpub() {
    let cosigner = Cosigner{
          xfp:Some("20F24288".to_string()),
          derivation_path: Some("m/48'/0'/0'/2'".to_string()),
          xpub: "zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi".to_string(),
        };

    let result = cosigner.descriptor(false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("xpub is not a multisig xpub. xpub:zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi"));
  }

  #[test]
  fn test_multisig_descriptor() {
    let cosigner1 = Cosigner{
          xfp:Some("20F24288".to_string()),
          derivation_path: Some("m/48'/0'/0'/2'".to_string()),
          xpub: "Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi".to_string(),
        };

    let cosigner2 = Cosigner{
          xfp:Some("E9A0CF4A".to_string()),
          derivation_path: Some("m/48'/0'/0'/2'".to_string()),
          xpub: "Zpub74kbYv5LXvBaJRcbSiihEEwuDiBSDztjtpSVmt6C6nB3ntbcEy4pLP3cJGVWsKbYKaAynfCwXnkuVncPGQ9Y4XwWJDWrDMUwTztdxBe7GcM".to_string(),
        };

    let cosigner3 = Cosigner{
          xfp:Some("232377FA".to_string()),
          derivation_path: Some("m/48'/0'/0'/2'".to_string()),
          xpub: "Zpub75QjBtEjDZJyE1f494V7uQqy4seryeCKQVGQQSCjutDt1WmwCyBMcY5AgxSjZwgfzBm6V2SULe5DvDLi1n3wHGtrrbsmjd1FDZ938VqVQuC".to_string(),
        };
    let mut multisig = Multisig::new(2);
    multisig.add_cosigner(cosigner1);
    multisig.add_cosigner(cosigner2);

    assert_eq!(multisig.descriptor(false).unwrap(), "wsh(sortedmulti(2,[20F24288/48'/0'/0'/2']xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/0/*,[E9A0CF4A/48'/0'/0'/2']xpub6EBypM14fbYFBG4yqLgTyzR69yrH8QE9kKkPHq2sxzat4WQEyFMR18sQT9csK54oGpsPR81hjFfJc3mVXnAZKaTdj51be8Ny2pVUb3jv6MC/0/*))");
    multisig.add_cosigner(cosigner3);

    assert_eq!(multisig.descriptor(false).unwrap(), "wsh(sortedmulti(2,[20F24288/48'/0'/0'/2']xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/0/*,[E9A0CF4A/48'/0'/0'/2']xpub6EBypM14fbYFBG4yqLgTyzR69yrH8QE9kKkPHq2sxzat4WQEyFMR18sQT9csK54oGpsPR81hjFfJc3mVXnAZKaTdj51be8Ny2pVUb3jv6MC/0/*,[232377FA/48'/0'/0'/2']xpub6Er7TKATMEfe6r7SXgStfAKA19Kht3XjFzaHvP9Rn6diH8aZwFTxHHtxqqa61h9vwSTW7VFEY6yd2UVpHA4xYKQzHTNXAPuGnNjsmQqLoQB/0/*))");
    multisig.threshold = 3;
    assert_eq!(multisig.descriptor(true).unwrap(), "wsh(sortedmulti(3,[20F24288/48'/0'/0'/2']xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/1/*,[E9A0CF4A/48'/0'/0'/2']xpub6EBypM14fbYFBG4yqLgTyzR69yrH8QE9kKkPHq2sxzat4WQEyFMR18sQT9csK54oGpsPR81hjFfJc3mVXnAZKaTdj51be8Ny2pVUb3jv6MC/1/*,[232377FA/48'/0'/0'/2']xpub6Er7TKATMEfe6r7SXgStfAKA19Kht3XjFzaHvP9Rn6diH8aZwFTxHHtxqqa61h9vwSTW7VFEY6yd2UVpHA4xYKQzHTNXAPuGnNjsmQqLoQB/1/*))");
  }

  #[test]
  fn test_multisig_descriptor_should_fail_for_threshold_greater_than_signers() {
    let cosigner1 = Cosigner{
          xfp:Some("20F24288".to_string()),
          derivation_path: Some("m/48'/0'/0'/2'".to_string()),
          xpub: "Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi".to_string(),
        };

    let cosigner2 = Cosigner{
          xfp:Some("E9A0CF4A".to_string()),
          derivation_path: Some("m/48'/0'/0'/2'".to_string()),
          xpub: "Zpub74kbYv5LXvBaJRcbSiihEEwuDiBSDztjtpSVmt6C6nB3ntbcEy4pLP3cJGVWsKbYKaAynfCwXnkuVncPGQ9Y4XwWJDWrDMUwTztdxBe7GcM".to_string(),
        };
    let mut cosigners = Multisig::new(3);
    cosigners.add_cosigner(cosigner1);
    cosigners.add_cosigner(cosigner2);
    let result = cosigners.descriptor(false);
    assert!(result.is_err());
    assert_eq!(
      result.unwrap_err().to_string(),
      "multisig threshold: 3 is greater than the number of cosigners: 2"
    );
  }

  #[test]
  fn test_wallet_get_descriptors() {
    let cosigner1 = Cosigner{
          xfp:Some("20F24288".to_string()),
          derivation_path: Some("m/48'/0'/0'/2'".to_string()),
          xpub: "Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi".to_string(),
        };

    let cosigner2 = Cosigner{
          xfp:Some("E9A0CF4A".to_string()),
          derivation_path: Some("m/48'/0'/0'/2'".to_string()),
          xpub: "Zpub74kbYv5LXvBaJRcbSiihEEwuDiBSDztjtpSVmt6C6nB3ntbcEy4pLP3cJGVWsKbYKaAynfCwXnkuVncPGQ9Y4XwWJDWrDMUwTztdxBe7GcM".to_string(),
        };
    let mut multisig = Multisig::new(2);
    multisig.add_cosigner(cosigner1);
    multisig.add_cosigner(cosigner2);

    let blockchain = Blockchain::new(
      "ssl://electrum.blockstream.info:60002",
      bitcoin::Network::Bitcoin,
    )
    .unwrap();
    let wallet = Wallet::from_multisig(&blockchain, &multisig).unwrap();
    let descriptors = wallet.get_descriptors().unwrap();
    assert_eq!(descriptors.descriptor, "wsh(sortedmulti(2,[20f24288/48'/0'/0'/2']xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/0/*,[e9a0cf4a/48'/0'/0'/2']xpub6EBypM14fbYFBG4yqLgTyzR69yrH8QE9kKkPHq2sxzat4WQEyFMR18sQT9csK54oGpsPR81hjFfJc3mVXnAZKaTdj51be8Ny2pVUb3jv6MC/0/*))#8dwny30k");
    assert_eq!(descriptors.change_descriptor, Some("wsh(sortedmulti(2,[20f24288/48'/0'/0'/2']xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/1/*,[e9a0cf4a/48'/0'/0'/2']xpub6EBypM14fbYFBG4yqLgTyzR69yrH8QE9kKkPHq2sxzat4WQEyFMR18sQT9csK54oGpsPR81hjFfJc3mVXnAZKaTdj51be8Ny2pVUb3jv6MC/1/*))#77ah2z6r".to_string()));
  }

  #[test]
  fn test_wallet_get_new_address() {
    let cosigner1 = Cosigner{
          xfp:Some("20F24288".to_string()),
          derivation_path: Some("m/48'/0'/0'/2'".to_string()),
          xpub: "Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi".to_string(),
        };

    let cosigner2 = Cosigner{
          xfp:Some("E9A0CF4A".to_string()),
          derivation_path: Some("m/48'/0'/0'/2'".to_string()),
          xpub: "Zpub74kbYv5LXvBaJRcbSiihEEwuDiBSDztjtpSVmt6C6nB3ntbcEy4pLP3cJGVWsKbYKaAynfCwXnkuVncPGQ9Y4XwWJDWrDMUwTztdxBe7GcM".to_string(),
        };
    let mut multisig = Multisig::new(2);
    multisig.add_cosigner(cosigner1);
    multisig.add_cosigner(cosigner2);

    let blockchain = Blockchain::new(
      "ssl://electrum.blockstream.info:60002",
      bitcoin::Network::Bitcoin,
    )
    .unwrap();
    let wallet = Wallet::from_multisig(&blockchain, &multisig).unwrap();
    let address = wallet.get_new_address().unwrap();
    assert_eq!(
      address.to_string(),
      "bc1q0gepljl6qsn9cn5d3z3jdvsrvrj4xr0j36g8tdhkkqdsw5jmxz2qdfkns2"
    );
    //Calling it again should return same address since it has not been used
    let address = wallet.get_new_address().unwrap();
    assert_eq!(
      address.to_string(),
      "bc1q0gepljl6qsn9cn5d3z3jdvsrvrj4xr0j36g8tdhkkqdsw5jmxz2qdfkns2"
    );
  }

  #[test]
  fn test_wallet_build_tx_encoded() {
    let cosigner1 = Cosigner{
      xfp:None,
      derivation_path: None,
      xpub: "Vpub5fCyVFyiBup7VKTCTX1vrMP4h2rjz8mxkmwzS8PZ1hZVf3U1AhKU49AYkom3KXDS4jLNyvnvobWkpkESVT3n8RkpwKWCBcUiV3y7wFFRktE".to_string(),
    };

    let cosigner2 = Cosigner{
      xfp:None,
      derivation_path: None,
      xpub: "Vpub5fL9LPRhSA6g5b56Dbgo6ruJ1DASCWE12jwiViwygh6NL3ALDoXVQpsiRApMSjvfotEKpSW9J2RgXoJLvDni2mT9mYicKrisXzQaMGBJfYv".to_string(),
    };
    let mut multisig = Multisig::new(2);
    multisig.add_cosigner(cosigner1);
    multisig.add_cosigner(cosigner2);

    let blockchain = Blockchain::new(
      "ssl://electrum.blockstream.info:60002",
      bitcoin::Network::Testnet,
    )
    .unwrap();
    let wallet = Wallet::from_multisig(&blockchain, &multisig).unwrap();
    let trx = Trx {
      descriptors: wallet.get_descriptors().unwrap(),
      to_address:"tb1qhfku74zsrhvre7053xqsnh36gsey3ur7slwwnfn04g5506rmdchqrf7w30".to_string(),
      amount: 10_000,
      fee_sat_per_vb: 5.0,
    };
    println!("Trx: {:#?}", trx);
    println!("Address: {:#?}", wallet.get_new_address().unwrap());
    println!("Balance: {:#?}", wallet.get_balance().unwrap());
    wallet.build_tx_encoded(&trx).unwrap();
  }

  #[test]
  fn test_address() {
    let address = Address::from_str("tb1qhfku74zsrhvre7053xqsnh36gsey3ur7slwwnfn04g5506rmdchqrf7w30").unwrap();
    assert_eq!(address.to_string(), "tb1qhfku74zsrhvre7053xqsnh36gsey3ur7slwwnfn04g5506rmdchqrf7w30");
    assert_eq!(address.address_type().unwrap(), AddressType::P2wsh);

    let address = Address::from_str("tb1q0tuqrsd5tqqa9l3juwn9un7lgax7u3mg6uzglf").unwrap();
    assert_eq!(address.to_string(), "tb1q0tuqrsd5tqqa9l3juwn9un7lgax7u3mg6uzglf");
    assert_eq!(address.address_type().unwrap(), AddressType::P2wpkh);
  }
}
