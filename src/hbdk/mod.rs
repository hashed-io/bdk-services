pub mod util;
pub mod errors;

use bdk::blockchain::ElectrumBlockchain;
use bdk::database::MemoryDatabase;
use bdk::electrum_client::Client;
use bdk::wallet::export::WalletExport;
use bdk::wallet::AddressIndex;
use bdk::{FeeRate, KeychainKind, SignOptions, SyncOptions};
use bitcoin::consensus::{deserialize, serialize};
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::Network;
use std::error;
use rocket::serde::{Deserialize, Serialize, json::Json};
use errors::Error;


type BoxResult<T> = Result<T,Box<dyn error::Error>>;

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
      return Err(bdk::Error::Generic(format!("xpub is not a multisig xpub. xpub:{}",self.xpub)).into())
    }
    Ok(format!("{}{}{}", s, util::to_legacy_xpub(&self.xpub)? , child))
  }
}

#[derive(Deserialize, Serialize)]
pub struct Multisig {
  pub threshold: u32,
  pub cosigners: Vec<Cosigner>,
}

impl Multisig {

  pub fn new(threshold: u32) -> Self{
    return Multisig {
      threshold,
      cosigners: Vec::new()
    }
  }

  pub fn add_cosigner(&mut self, cosigner: Cosigner){
    self.cosigners.push(cosigner);
  }

  pub fn descriptor(&self, change: bool) -> Result<String, Error> {
    if self.threshold > self.cosigners.len() as u32 {
      return Err(errors::Error::new(&format!("multisig threshold: {} is greater than the number of cosigners: {}", self.threshold, self.cosigners.len())).into())
    }
    let mut descriptor = String::new();
    for cosigner in &self.cosigners {
      descriptor = format!("{},{}", descriptor,cosigner.descriptor(change)?);
    }
    Ok(format!("wsh(sortedmulti({}{}))", self.threshold, descriptor))
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Descriptors {
  pub descriptor: String,
  pub change_descriptor: String,
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
}

pub struct Wallet {
  blockchain: Blockchain,
  wallet: bdk::Wallet<MemoryDatabase>,
}

impl Wallet {
  pub fn from_multisig(blockchain: Blockchain, multisig: &Multisig) -> Result<Self, Error> {
    Self::from_descriptors(
      blockchain,
      &multisig.descriptor(false)?,
      &multisig.descriptor(true)?,
    )
  }

  pub fn from_descriptors(
    blockchain: Blockchain,
    descriptor: &str,
    change_descriptor: &str,
  ) -> Result<Self, Error> {
    let network = blockchain.network.clone();
    return Ok(Wallet {
      blockchain,
      wallet: bdk::Wallet::new(
        descriptor,
        Some(change_descriptor),
        network,
        MemoryDatabase::default(),
      )?,
    });
  }

  pub fn get_descriptors(&self) -> Result<Descriptors, Error> {
    let descriptor = self
      .wallet
      .public_descriptor(KeychainKind::External)?
      .ok_or(bdk::Error::Generic("No descriptor for wallet".to_string()))?;
    let change_descriptor = self
      .wallet
      .public_descriptor(KeychainKind::Internal)?
      .ok_or(bdk::Error::Generic(
        "No change descriptor for wallet".to_string(),
      ))?;
    return Ok(Descriptors {
      descriptor: format!("{}", descriptor),
      change_descriptor: format!("{}", change_descriptor),
    });
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
    assert_eq!(result.unwrap_err().to_string(), "multisig threshold: 3 is greater than the number of cosigners: 2");
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

    let blockchain = Blockchain::new("ssl://electrum.blockstream.info:60002",bitcoin::Network::Bitcoin).unwrap();
    let wallet = Wallet::from_multisig(blockchain, &multisig).unwrap();
    let descriptors = wallet.get_descriptors().unwrap();
    assert_eq!(descriptors.descriptor, "wsh(sortedmulti(2,[20f24288/48'/0'/0'/2']xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/0/*,[e9a0cf4a/48'/0'/0'/2']xpub6EBypM14fbYFBG4yqLgTyzR69yrH8QE9kKkPHq2sxzat4WQEyFMR18sQT9csK54oGpsPR81hjFfJc3mVXnAZKaTdj51be8Ny2pVUb3jv6MC/0/*))#8dwny30k"); 
    assert_eq!(descriptors.change_descriptor, "wsh(sortedmulti(2,[20f24288/48'/0'/0'/2']xpub6F2hcB5PLR3L7BSjEeVCqCho8oC7m23U5jF48mkwEPdn5zmhmayu6tSPacmGUuG4LU1rJT7sRr6QJz8mrVkTCadMCaQHisQFQrP4y1uRvYH/1/*,[e9a0cf4a/48'/0'/0'/2']xpub6EBypM14fbYFBG4yqLgTyzR69yrH8QE9kKkPHq2sxzat4WQEyFMR18sQT9csK54oGpsPR81hjFfJc3mVXnAZKaTdj51be8Ny2pVUb3jv6MC/1/*))#77ah2z6r"); 
  }
}
