pub mod errors;
pub mod util;

use bdk::blockchain::{Blockchain as BlockchainTrait, ElectrumBlockchain};
use bdk::database::MemoryDatabase;
use bdk::descriptor::Descriptor;
use bdk::electrum_client::Client;
use bdk::wallet::{AddressIndex, AddressInfo};
use bdk::{BlockTime, FeeRate, KeychainKind, SignOptions, SyncOptions, TransactionDetails};
use bitcoin::blockdata::{script::Script, transaction::OutPoint};
use bitcoin::consensus;
use bitcoin::util::address::Address;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::{hash_types::Txid, Network, Transaction};
use core::str::FromStr;
use errors::Error;
use lazy_static::lazy_static;
use miniscript::descriptor::{DescriptorPublicKey, WshInner};
use regex::Regex;
use rocket::serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::convert::TryFrom;

#[derive(Debug, Deserialize, Serialize, Hash)]
#[serde(try_from = "CosignerShadow")]
pub struct Cosigner {
  pub xfp: Option<String>,
  pub xpub: String,
  pub derivation_path: Option<String>,
}

#[derive(Deserialize)]
pub struct CosignerShadow {
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
      return Err(Error::new(&format!(
        "xpub is not a multisig xpub. xpub:{}",
        self.xpub
      )));
    }
    Ok(format!(
      "{}{}{}",
      s,
      util::to_legacy_xpub(&self.xpub)?,
      child
    ))
  }
}

impl TryFrom<CosignerShadow> for Cosigner {
  type Error = Error;

  fn try_from(shadow: CosignerShadow) -> Result<Self, Self::Error> {
    let mut cosigner = Cosigner::from_str(&shadow.xpub)?;
    if cosigner.xfp.is_none() {
      cosigner.xfp = shadow.xfp.map(|s| s.to_lowercase());
      cosigner.derivation_path = shadow.derivation_path;
    }
    Ok(cosigner)
  }
}

impl FromStr for Cosigner {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Error> {
    lazy_static! {
      static ref XPUB_RE: Regex = Regex::new(r"^[a-zA-Z\d]{111,112}$").unwrap();
      static ref FULL_XPUB_RE: Regex = Regex::new(
        r"^\[(?P<xfp>[a-zA-Z\d]{8})(?P<dp>(?:/\d*')+)\](?P<xpub>[a-zA-Z\d]{111,112})(?:/[\d\*]+)*$"
      )
      .unwrap();
    }
    if FULL_XPUB_RE.is_match(s) {
      let captures = FULL_XPUB_RE.captures(s).unwrap();
      return Ok(Cosigner {
        xfp: Some(String::from(
          captures.name("xfp").unwrap().as_str().to_lowercase(),
        )),
        xpub: String::from(captures.name("xpub").unwrap().as_str()),
        derivation_path: Some(format!("m{}", captures.name("dp").unwrap().as_str())),
      });
    } else if XPUB_RE.is_match(s) {
      return Ok(Cosigner {
        xfp: None,
        xpub: String::from(s),
        derivation_path: None,
      });
    }
    Err(Error::new(&format!("invalid xpub format, xpub:{}", s)))
  }
}

impl PartialEq for Cosigner {
  fn eq(&self, other: &Self) -> bool {
    if let Some(xfp) = &self.xfp {
      if other.xfp.is_none() {
        return false;
      }
      if xfp.to_lowercase() != other.xfp.as_ref().unwrap().to_lowercase() {
        return false;
      }
    } else {
      if other.xfp.is_some() {
        return false;
      }
    }
    self.derivation_path == other.derivation_path && self.xpub == other.xpub
  }
}

impl Eq for Cosigner {}

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
  pub broadcast: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TrxDetails {
  pub trx_id: Txid,
  /// Received value (sats)
  pub received: u64,
  /// Sent value (sats)
  pub sent: u64,
  /// Fee value (sats) if available.
  /// The availability of the fee depends on the backend. It's never `None` with an Electrum
  /// Server backend, but it could be `None` with a Bitcoin RPC node without txindex that receive
  /// funds while offline.
  pub fee: Option<u64>,
  /// If the transaction is confirmed, contains height and timestamp of the block containing the
  /// transaction, unconfirmed transaction contains `None`.
  pub confirmation_time: Option<BlockTime>,

  pub inputs: Vec<TrxInput>,
  pub outputs: Vec<TrxOutput>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TrxInput {
  previous_output_trx: OutPoint,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TrxOutput {
  value: u64,
  script_pubkey: Script,
  address: Option<Address>,
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

  pub fn broadcast(&self, tx: &Transaction) -> Result<(), Error> {
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
    let descriptor = self.get_external_descriptor()?;
    let change_descriptor = self.wallet.public_descriptor(KeychainKind::Internal)?;
    let change_descriptor = change_descriptor.map(|desc| desc.to_string());
    Ok(Descriptors {
      descriptor: descriptor.to_string(),
      change_descriptor,
    })
  }

  pub fn get_multisig(&self) -> Result<Multisig, Error> {
    let mut multisig;
    let descriptor = self.get_external_descriptor()?;
    if let Descriptor::Wsh(wsh) = descriptor {
      if let WshInner::SortedMulti(sm) = wsh.as_inner() {
        multisig = Multisig::new(sm.k as u32);
        for pk in &sm.pks {
          if let DescriptorPublicKey::XPub(xpub) = pk {
            let mut cosigner = Cosigner::from_str(&util::to_segwit_native_multisig_xpub(
              &xpub.xkey.to_string(),
            )?)?;
            if let Some((xfp, derivation_path)) = &xpub.origin {
              cosigner.xfp = Some(xfp.to_string());
              cosigner.derivation_path = Some(derivation_path.to_string());
            }
            multisig.add_cosigner(cosigner);
          } else {
            return Err(Error::new(&format!(
              "Wallet does not only contain xpubs, found:{}",
              pk
            )));
          }
        }
      } else {
        return Err(Error::new(
          "Wallet is not of type sorted multisig, found miniscript",
        ));
      }
    } else {
      return Err(Error::new(&format!(
        "Wallet is not of type Pay-to-Witness-Script-Hash, found: {}",
        descriptor
      )));
    }
    Ok(multisig)
  }

  pub fn get_new_address(&self) -> Result<AddressInfo, Error> {
    self.sync()?;
    let address = self.wallet.get_address(AddressIndex::LastUnused)?;
    Ok(address)
  }

  pub fn list_trxs(&self) -> Result<Vec<TrxDetails>, Error> {
    self.sync()?;
    let mut trxs: Vec<TrxDetails> = vec!();
    let original_trxs = self.wallet.list_transactions(true)?;
    for otrx in original_trxs {
      let mut inputs: Vec<TrxInput> = vec!();
      let mut outputs: Vec<TrxOutput> = vec!();
      if let Some(rtrx) = otrx.transaction {
        for input in rtrx.input {
          inputs.push(TrxInput{
            previous_output_trx: input.previous_output
          });
        }
        for output in rtrx.output {
          outputs.push(TrxOutput{
            address: Address::from_script(&output.script_pubkey, self.blockchain.network),
            script_pubkey: output.script_pubkey,
            value: output.value,
          });
        }

          // let addr = Address::from_script(&o.script_pubkey, self.blockchain.network).unwrap();
          // println!("address output: {}", addr);
        }
        trxs.push(TrxDetails{
          trx_id: otrx.txid,
          received: otrx.received,
          sent: otrx.sent,
          fee: otrx.fee,
          confirmation_time: otrx.confirmation_time,
          inputs: inputs,
          outputs: outputs,
        });
    }
    Ok(trxs)
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
      // .do_not_spend_change()
      .fee_rate(FeeRate::from_sat_per_vb(trx.fee_sat_per_vb));
    Ok(builder.finish()?)
  }

  pub fn build_tx_encoded(&self, trx: &Trx) -> Result<String, Error> {
    let (psbt, _) = self.build_tx(trx)?;
    Ok(base64::encode(consensus::serialize(&psbt)))
  }

  pub fn finalize_trx(&self, psbts: &[String], broadcast: bool) -> Result<String, Error> {
    if psbts.len() < 1 {
      return Err(Error::new(&format!(
        "failed to finalized tx, there are less than required psbts, found: {}",
        psbts.len()
      )));
    }
    let mut combined = self.deserialize_psbt(&psbts[0])?;
    for psbt in &psbts[1..] {
      combined.combine(self.deserialize_psbt(psbt)?)?;
    }
    self.sync()?;
    let finalized = self
      .wallet
      .finalize_psbt(&mut combined, SignOptions::default())?;
    if !finalized {
      return Err(Error::new("provided psbts do not finalize tx"));
    }
    let tx = combined.extract_tx();
    if broadcast {
      self.blockchain.broadcast(&tx)?;
    }
    Ok(tx.txid().to_string())
  }

  fn get_external_descriptor(
    &self,
  ) -> Result<miniscript::Descriptor<miniscript::DescriptorPublicKey>, Error> {
    let descriptor = self
      .wallet
      .public_descriptor(KeychainKind::External)?
      .ok_or(bdk::Error::Generic("No descriptor for wallet".to_string()))?;
    Ok(descriptor)
  }

  fn deserialize_psbt(&self, psbt: &str) -> Result<PartiallySignedTransaction, Error> {
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
  use bitcoin::util::address::AddressType;

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
  fn test_wallet_get_multisig() {
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
    let mut original_multisig = Multisig::new(2);
    original_multisig.add_cosigner(cosigner1);
    original_multisig.add_cosigner(cosigner2);

    let blockchain = Blockchain::new(
      "ssl://electrum.blockstream.info:60002",
      bitcoin::Network::Bitcoin,
    )
    .unwrap();
    let wallet = Wallet::from_multisig(&blockchain, &original_multisig).unwrap();
    assert_multisig(&mut original_multisig, &mut wallet.get_multisig().unwrap());

    let cosigner1 = Cosigner:: from_str("Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi").unwrap();
    let cosigner2 = Cosigner:: from_str("Zpub74kbYv5LXvBaJRcbSiihEEwuDiBSDztjtpSVmt6C6nB3ntbcEy4pLP3cJGVWsKbYKaAynfCwXnkuVncPGQ9Y4XwWJDWrDMUwTztdxBe7GcM").unwrap();

    let mut original_multisig = Multisig::new(2);
    original_multisig.add_cosigner(cosigner1);
    original_multisig.add_cosigner(cosigner2);

    let wallet = Wallet::from_multisig(&blockchain, &original_multisig).unwrap();
    assert_multisig(&mut original_multisig, &mut wallet.get_multisig().unwrap());
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
      to_address: "tb1qhfku74zsrhvre7053xqsnh36gsey3ur7slwwnfn04g5506rmdchqrf7w30".to_string(),
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
    let address =
      Address::from_str("tb1qhfku74zsrhvre7053xqsnh36gsey3ur7slwwnfn04g5506rmdchqrf7w30").unwrap();
    assert_eq!(
      address.to_string(),
      "tb1qhfku74zsrhvre7053xqsnh36gsey3ur7slwwnfn04g5506rmdchqrf7w30"
    );
    assert_eq!(address.address_type().unwrap(), AddressType::P2wsh);

    let address = Address::from_str("tb1q0tuqrsd5tqqa9l3juwn9un7lgax7u3mg6uzglf").unwrap();
    assert_eq!(
      address.to_string(),
      "tb1q0tuqrsd5tqqa9l3juwn9un7lgax7u3mg6uzglf"
    );
    assert_eq!(address.address_type().unwrap(), AddressType::P2wpkh);
  }

  #[test]
  fn test_cosigner_from_str_for_full_xpub() {
    let cosigner =
      Cosigner::from_str("[0CDB4EE2/48'/0'/0'/2']Zpub753WkfemgkpJqtboFVaoqHqBSVEQNgEdKmpRuMkNNabVv6ATumRRhNUdrnQopkgLnAxwZxzkh7rDvsCoEvBHuKuojKtSFfuroukMw9Kv1Ui/1/*").unwrap();

    assert_eq!(cosigner.xfp, Some(String::from("0cdb4ee2")));
    assert_eq!(
      cosigner.derivation_path,
      Some(String::from("m/48'/0'/0'/2'"))
    );
    assert_eq!(
      cosigner.xpub,
      "Zpub753WkfemgkpJqtboFVaoqHqBSVEQNgEdKmpRuMkNNabVv6ATumRRhNUdrnQopkgLnAxwZxzkh7rDvsCoEvBHuKuojKtSFfuroukMw9Kv1Ui"      
    );
  }

  #[test]
  fn test_cosigner_from_str_for_xpub() {
    let cosigner =
      Cosigner::from_str("Zpub753WkfemgkpJqtboFVaoqHqBSVEQNgEdKmpRuMkNNabVv6ATumRRhNUdrnQopkgLnAxwZxzkh7rDvsCoEvBHuKuojKtSFfuroukMw9Kv1Ui").unwrap();

    assert_eq!(cosigner.xfp, None);
    assert_eq!(cosigner.derivation_path, None);
    assert_eq!(
      cosigner.xpub,
      "Zpub753WkfemgkpJqtboFVaoqHqBSVEQNgEdKmpRuMkNNabVv6ATumRRhNUdrnQopkgLnAxwZxzkh7rDvsCoEvBHuKuojKtSFfuroukMw9Kv1Ui"      
    );
  }

  #[test]
  #[should_panic(expected = "invalid xpub format, xpub")]
  fn test_cosigner_from_str_should_fail_for_invalid_xpub() {
    Cosigner::from_str("[asd]Zpub753WkfemgkpJqtboFVaoqHqBSVEQNgEdKmpRuMkNNabVv6ATumRRhNUdrnQopkgLnAxwZxzkh7rDvsCoEvBHuKuojKtSFfuroukMw9Kv1Ui").unwrap();
  }

  fn assert_multisig(ms1: &mut Multisig, ms2: &mut Multisig) {
    assert_eq!(ms1.threshold, ms2.threshold);
    assert_cosigners(&mut ms1.cosigners, &mut ms2.cosigners);
  }

  fn assert_cosigners(css1: &mut Vec<Cosigner>, css2: &mut Vec<Cosigner>) {
    assert_eq!(
      css1.len(),
      css2.len(),
      "Different number of cosigners, cs1: {}, cs2: {}",
      css1.len(),
      css2.len()
    );
    let mut iter1 = css1.iter_mut();
    for cs1 in css2 {
      let rcs2 = iter1.find(|cs2| {
        return cs1.xpub == cs2.xpub;
      });
      assert!(rcs2.is_some(), "Cosigner not found: {:?}", cs1);
      assert_eq!(cs1, rcs2.unwrap());
    }
  }

  #[test]
  fn test_cosigner_try_from_cosigner_shadow_for_xpub_with_full_xpub() {
    let shadow = CosignerShadow {
      xfp: None,
      xpub: String::from("[0CDB4EE2/48'/0'/0'/2']Zpub753WkfemgkpJqtboFVaoqHqBSVEQNgEdKmpRuMkNNabVv6ATumRRhNUdrnQopkgLnAxwZxzkh7rDvsCoEvBHuKuojKtSFfuroukMw9Kv1Ui/1/*"),
      derivation_path: None
    };
    let cosigner = Cosigner::try_from(shadow).unwrap();

    assert_eq!(cosigner.xfp, Some(String::from("0cdb4ee2")));
    assert_eq!(
      cosigner.derivation_path,
      Some(String::from("m/48'/0'/0'/2'"))
    );
    assert_eq!(
      cosigner.xpub,
      "Zpub753WkfemgkpJqtboFVaoqHqBSVEQNgEdKmpRuMkNNabVv6ATumRRhNUdrnQopkgLnAxwZxzkh7rDvsCoEvBHuKuojKtSFfuroukMw9Kv1Ui"      
    );
  }

  #[test]
  fn test_cosigner_try_from_cosigner_shadow_for_xpub_with_xpub() {
    let shadow = CosignerShadow {
      xfp: None,
      xpub: String::from("Zpub753WkfemgkpJqtboFVaoqHqBSVEQNgEdKmpRuMkNNabVv6ATumRRhNUdrnQopkgLnAxwZxzkh7rDvsCoEvBHuKuojKtSFfuroukMw9Kv1Ui"),
      derivation_path: None
    };
    let cosigner = Cosigner::try_from(shadow).unwrap();

    assert_eq!(cosigner.xfp, None);
    assert_eq!(cosigner.derivation_path, None);
    assert_eq!(
      cosigner.xpub,
      "Zpub753WkfemgkpJqtboFVaoqHqBSVEQNgEdKmpRuMkNNabVv6ATumRRhNUdrnQopkgLnAxwZxzkh7rDvsCoEvBHuKuojKtSFfuroukMw9Kv1Ui"      
    );
  }

  #[test]
  fn test_cosigner_try_from_cosigner_shadow_for_all_properties_set() {
    let shadow = CosignerShadow {
      xfp: Some(String::from("0CDB4EE2")),
      xpub: String::from("Zpub753WkfemgkpJqtboFVaoqHqBSVEQNgEdKmpRuMkNNabVv6ATumRRhNUdrnQopkgLnAxwZxzkh7rDvsCoEvBHuKuojKtSFfuroukMw9Kv1Ui"),
      derivation_path: Some(String::from("m/48'/0'/0'/2'"))
    };
    let cosigner = Cosigner::try_from(shadow).unwrap();

    assert_eq!(cosigner.xfp, Some(String::from("0cdb4ee2")));
    assert_eq!(
      cosigner.derivation_path,
      Some(String::from("m/48'/0'/0'/2'"))
    );
    assert_eq!(
      cosigner.xpub,
      "Zpub753WkfemgkpJqtboFVaoqHqBSVEQNgEdKmpRuMkNNabVv6ATumRRhNUdrnQopkgLnAxwZxzkh7rDvsCoEvBHuKuojKtSFfuroukMw9Kv1Ui"      
    );
  }

  #[test]
  #[should_panic(expected = "invalid xpub format, xpub")]
  fn test_cosigner_try_from_cosigner_shadow_should_fail_for_invalid_xpub() {
    let shadow = CosignerShadow {
      xfp: None,
      xpub: String::from("[asda]Zpub753WkfemgkpJqtboFVaoqHqBSVEQNgEdKmpRuMkNNabVv6ATumRRhNUdrnQopkgLnAxwZxzkh7rDvsCoEvBHuKuojKtSFfuroukMw9Kv1Ui"),
      derivation_path: None
    };
    Cosigner::try_from(shadow).unwrap();
  }
}
