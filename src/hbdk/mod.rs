pub mod errors;
pub mod util;

use bdk::blockchain::{Blockchain as BlockchainTrait, ElectrumBlockchain};
use bdk::database::{Database, MemoryDatabase};
use bdk::descriptor::{Descriptor, DescriptorPublicKey};
use bdk::electrum_client::Client;
use bdk::miniscript::descriptor::WshInner;
use bdk::wallet::{AddressIndex, AddressInfo};
use bdk::{BlockTime, FeeRate, KeychainKind, SignOptions, SyncOptions, TransactionDetails};
use bdk_reserves::reserves::ProofOfReserves as ProofOfReservesTrait;
use bitcoin::blockdata::{script::Script, transaction::OutPoint};
use bitcoin::util::address::Address;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::{consensus, psbt, LockTime, Sequence};
use bitcoin::{
    hash_types::Txid,
    secp256k1::{All, Secp256k1},
    Network, Transaction,
};
use core::{ops::Deref, str::FromStr};
use errors::Error;
use lazy_static::lazy_static;

use miniscript::interpreter::Interpreter;
use regex::Regex;
use rocket::serde::{Deserialize, Serialize};
use std::{clone::Clone, collections::BTreeMap, convert::TryFrom};

#[derive(Debug, Deserialize, Serialize, Hash, Clone)]
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
    pub fn public_key_descriptor(&self, change: bool) -> Result<DescriptorPublicKey, Error> {
        Ok(DescriptorPublicKey::from_str(&self.descriptor(change)?)?)
    }

    pub fn xfp_is(&self, fp: &str) -> bool {
        return if let Some(xfp) = &self.xfp {
            fp.to_lowercase() == xfp.to_lowercase()
        } else {
            false
        };
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

pub struct CosignerFinder<'a> {
    cosigners: Vec<&'a Cosigner>,
    pub_key_cosigner_map: Option<BTreeMap<bitcoin::PublicKey, &'a Cosigner>>,
    secp_ctx: &'a Secp256k1<All>,
    start_idx: u32,
    end_idx: u32,
}

impl<'a> CosignerFinder<'a> {
    pub fn new(
        cosigners: &'a Vec<Cosigner>,
        start_idx: u32,
        end_idx: u32,
        secp_ctx: &'a Secp256k1<All>,
    ) -> Self {
        assert!(start_idx < end_idx, "start_idx must be less than end_idx");
        CosignerFinder {
            cosigners: cosigners.iter().map(|cosigner| cosigner).collect(),
            pub_key_cosigner_map: None,
            secp_ctx,
            start_idx,
            end_idx,
        }
    }
    fn generate_pub_key_cosigner_map(
        &self,
    ) -> Result<BTreeMap<bitcoin::PublicKey, &'a Cosigner>, Error> {
        let mut map = BTreeMap::new();
        for cosigner in &self.cosigners {
            let xpub = cosigner.public_key_descriptor(true)?;
            for i in self.start_idx..self.end_idx {
                map.insert(
                    xpub.clone()
                        .at_derivation_index(i)
                        .derive_public_key(self.secp_ctx)?,
                    cosigner.clone(),
                );
            }
        }
        Ok(map)
    }

    pub fn find_by_public_key(
        &mut self,
        public_key: &bitcoin::PublicKey,
    ) -> Result<Option<&&'a Cosigner>, Error> {
        if self.pub_key_cosigner_map.is_none() {
            self.pub_key_cosigner_map = Some(self.generate_pub_key_cosigner_map()?);
        }
        let map = &self.pub_key_cosigner_map.as_ref().unwrap();
        Ok(map.get(public_key))
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
            return Err(errors::Error::new(&format!(
                "multisig threshold: {} is greater than the number of cosigners: {}",
                self.threshold,
                self.cosigners.len()
            ))
            .into());
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

    pub fn find_by_xfp(&self, xfp: &str) -> Option<&Cosigner> {
        self.cosigners.iter().find(|cosigner| cosigner.xfp_is(xfp))
    }

    pub fn public_key_descriptors(
        &self,
        change: bool,
    ) -> Result<Vec<(&Cosigner, DescriptorPublicKey)>, Error> {
        let mut xpubs = Vec::new();
        for cosigner in &self.cosigners {
            xpubs.push((cosigner, cosigner.public_key_descriptor(change)?));
        }
        Ok(xpubs)
    }

    pub fn cosigner_finder<'secp>(
        &'secp self,
        start_idx: u32,
        end_idx: u32,
        secp_ctx: &'secp Secp256k1<All>,
    ) -> CosignerFinder {
        CosignerFinder::new(&self.cosigners, start_idx, end_idx, secp_ctx)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VerifyPSBTPayload {
    pub descriptors: Descriptors,
    pub psbt: String,
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
pub struct SignedProofOfReserves {
    pub descriptors: Descriptors,
    pub psbts: Vec<String>,
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

#[derive(Deserialize, Serialize, Debug)]
pub struct ProofOfReservesRequest {
    pub descriptors: Descriptors,
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProofOfReserves {
    pub descriptors: Descriptors,
    pub psbt: String,
    pub message: String,
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
                        let mut cosigner = Cosigner::from_str(
                            &util::to_segwit_native_multisig_xpub(&xpub.xkey.to_string())?,
                        )?;
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
        let mut trxs: Vec<TrxDetails> = vec![];
        let original_trxs = self.wallet.list_transactions(true)?;
        for otrx in original_trxs {
            let mut inputs: Vec<TrxInput> = vec![];
            let mut outputs: Vec<TrxOutput> = vec![];
            if let Some(rtrx) = otrx.transaction {
                for input in rtrx.input {
                    inputs.push(TrxInput {
                        previous_output_trx: input.previous_output,
                    });
                }
                for output in rtrx.output {
                    outputs.push(TrxOutput {
                        address: Some(Address::from_script(
                            &output.script_pubkey,
                            self.blockchain.network,
                        )?),
                        script_pubkey: output.script_pubkey,
                        value: output.value,
                    });
                }

                // let addr = Address::from_script(&o.script_pubkey, self.blockchain.network).unwrap();
                // println!("address output: {}", addr);
            }
            trxs.push(TrxDetails {
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
        println!("Balance: {:?}", balance);
        Ok(balance.get_total())
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
        Ok(self.serialize_psbt(&psbt))
    }

    pub fn get_signers(
        &self,
        serialized_psbt: &str,
        search_radius: u8,
    ) -> Result<Vec<Cosigner>, Error> {
        let mut cosigners = Vec::new();
        let psbt = self.deserialize_psbt(serialized_psbt)?;
        let multisig = self.get_multisig()?;
        for (i, input) in psbt.inputs.iter().enumerate() {
            if !input.partial_sigs.is_empty() {
                cosigners.append(&mut self.get_signers_from_partial_sigs(&input, &multisig)?);
            } else if input.final_script_witness.is_some() {
                cosigners.append(&mut self.get_signers_from_script_witness(
                    &psbt,
                    i,
                    &multisig,
                    search_radius,
                )?);
            }
        }
        Ok(cosigners)
    }

    fn get_signers_from_partial_sigs(
        &self,
        input: &psbt::Input,
        multisig: &Multisig,
    ) -> Result<Vec<Cosigner>, Error> {
        let mut cosigners = Vec::new();
        for (key, _) in input.partial_sigs.iter() {
            let keysource = input.bip32_derivation.get(&key.inner).unwrap();
            if let Some(cosigner) = multisig.find_by_xfp(&keysource.0.to_string()) {
                cosigners.push((*cosigner).clone());
            } else {
                return Err(Error::new(&format!(
                    "no cosigner found with finger print: {}",
                    &keysource.0.to_string()
                )));
            }
        }
        Ok(cosigners)
    }

    fn get_signers_from_script_witness(
        &self,
        psbt: &PartiallySignedTransaction,
        input_index: usize,
        multisig: &Multisig,
        search_radius: u8,
    ) -> Result<Vec<Cosigner>, Error> {
        self.sync()?;
        let mut cosigners = Vec::new();
        let index = self
            .get_last_derivation_index(KeychainKind::Internal)?
            .unwrap_or(0);
        let start_idx = std::cmp::max(0, index as i32 - search_radius as i32) as u32;
        let end_idx = index + search_radius as u32;
        let interpreter = self.get_tx_interpreter(psbt, input_index)?;
        let mut finder = multisig.cosigner_finder(start_idx, end_idx, self.wallet.secp_ctx());
        for elem in interpreter.iter_assume_sigs() {
            match elem.expect("no evaluation error") {
                miniscript::interpreter::SatisfiedConstraint::PublicKey { key_sig } => {
                    let (key, _) = key_sig
                        .as_ecdsa()
                        .expect("expected ecdsa sig, found schnorr sig");
                    println!("Signed with:\n key: {}\n", key);
                    let cosigner = finder.find_by_public_key(&key)?.ok_or(Error::new(&format!(
                        "no cosigner found for public key: {}",
                        key
                    )))?;
                    cosigners.push((*cosigner).clone());
                }
                _ => {}
            }
        }
        Ok(cosigners)
    }

    fn get_last_derivation_index(&self, keychain: KeychainKind) -> Result<Option<u32>, bdk::Error> {
        self.wallet.database().deref().get_last_index(keychain)
    }

    fn get_tx_interpreter<'psbt>(
        &'psbt self,
        psbt: &'psbt PartiallySignedTransaction,
        input_index: usize,
    ) -> Result<Interpreter, Error> {
        let script_sig = &psbt.unsigned_tx.input[input_index].script_sig;
        let input = &psbt.inputs[input_index];
        if let Some(witness_utxo) = &input.witness_utxo {
            if let Some(script_witness) = &input.final_script_witness {
                Ok(Interpreter::from_txdata(
                    &witness_utxo.script_pubkey,
                    &script_sig,
                    &script_witness,
                    Sequence(0),
                    LockTime::from_time(0)?,
                )?)
            } else {
                Err(Error::new(&format!(
                    "failed to create the tx interpreter, final script witness not found"
                )))
            }
        } else {
            Err(Error::new(&format!(
                "failed to create the tx interpreter, witness utxo not found"
            )))
        }
    }

    pub fn finalize_trx(&self, psbts: &[String], broadcast: bool) -> Result<String, Error> {
        let tx = self.finalize_psbt(psbts)?.extract_tx();
        if broadcast {
            self.blockchain.broadcast(&tx)?;
        }
        Ok(tx.txid().to_string())
    }

    fn finalize_psbt(&self, psbts: &[String]) -> Result<PartiallySignedTransaction, Error> {
        if psbts.len() < 1 {
            return Err(Error::new(&format!(
                "failed to finalized psbt, there are less than required psbts, found: {}",
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
        Ok(combined)
    }

    pub fn create_proof_of_reserves_encoded(&self, message: &str) -> Result<String, Error> {
        Ok(self.serialize_psbt(&self.create_proof_of_reserves(message)?))
    }

    pub fn create_proof_of_reserves(
        &self,
        message: &str,
    ) -> Result<PartiallySignedTransaction, Error> {
        self.sync()?;
        Ok(self.wallet.create_proof(message)?)
    }

    pub fn finalize_proof_of_reserves(&self, psbts: &[String]) -> Result<String, Error> {
        if psbts.len() < 1 {
            return Err(Error::new(&format!(
                "failed to finalized psbt, there are less than required psbts, found: {}",
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
        Ok(self.serialize_psbt(&combined))
    }

    pub fn verify_proof_of_reserves(&self, message: &str, psbt: &str) -> Result<u64, Error> {
        self.sync()?;
        Ok(self
            .wallet
            .verify_proof(&self.deserialize_psbt(psbt)?, message, None)?)
    }

    fn get_external_descriptor(&self) -> Result<Descriptor<DescriptorPublicKey>, Error> {
        let descriptor = self
            .wallet
            .public_descriptor(KeychainKind::External)?
            .ok_or(bdk::Error::Generic("No descriptor for wallet".to_string()))?;
        Ok(descriptor)
    }

    fn serialize_psbt(&self, psbt: &PartiallySignedTransaction) -> String {
        base64::encode(consensus::serialize(psbt))
    }

    fn deserialize_psbt(&self, psbt: &str) -> Result<PartiallySignedTransaction, Error> {
        let decoded = base64::decode(psbt).unwrap_or(psbt.as_bytes().to_vec());
        let deserialized = consensus::deserialize(&decoded)?;
        Ok(deserialized)
    }

    fn sync(&self) -> Result<(), Error> {
        self.wallet
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
            to_address: "tb1qhfku74zsrhvre7053xqsnh36gsey3ur7slwwnfn04g5506rmdchqrf7w30"
                .to_string(),
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
            Address::from_str("tb1qhfku74zsrhvre7053xqsnh36gsey3ur7slwwnfn04g5506rmdchqrf7w30")
                .unwrap();
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

    #[test]
    fn test_wallet_get_signers_no_signers() {
        let multisig = get_test_multisig();
        let blockchain = get_blockchain();
        let wallet = Wallet::from_multisig(&blockchain, &multisig).unwrap();
        let psbt = "cHNidP8BAIkBAAAAAfx15Ttmz6elm9LHqX2jVvqboFTMUrD3OVilRE0RH3HNAQAAAAD9////AhAnAAAAAAAAIgAgapL4iNK+iOvUjmi74v5KOdJq0+brS2MsQt8bZu/jvVy37QAAAAAAACIAIDGu4FBMXgV+irxy6Vz78NrpoH/ezv1eabyuP2wfZkIWAAAAAAABAOoCAAAAAAEBP6iqyBDPPxR5o7wsklupWR/BZ5huKtUVwLZusOOQqXoBAAAAAP7///8CqwsoMAAAAAAWABSESQw62sXLgOdVekMYn+qAywbPk0AZAQAAAAAAIgAgmvQof5tQQVQRoQxX6GSovQjs0nC5cuKt40qMf8DoDuUCRzBEAiAkJRaqNQ3sbtMc4klUmqVekYtPklFhARt6lheow5JaggIgF/toSoiQluyUS2D0nPZ0aiikrAjjVLkRDRPPZhdbeVsBIQPVQZS4ubQQ+Sao6+dX9Em/At/0S41/SGBpzvniCo8K4FTYIgABAStAGQEAAAAAACIAIJr0KH+bUEFUEaEMV+hkqL0I7NJwuXLireNKjH/A6A7lAQWLUyECud7M+qn/i4LyHa0YVFUTevc6HBCmEhIqgM5VUADJYuYhA0cI+7c8yLZFJUQmXcefoV2FihbENMR29W+DQjZX78krIQNm7ihjOrXbPXri3XycoPFJeq7IpFN3Mgx9prkdzwTx6CED9Yk2xtBbhmmpBzjCAlfHvo3Qn0Fe2a2fwpLoIaedVk9UriIGArnezPqp/4uC8h2tGFRVE3r3OhwQphISKoDOVVAAyWLmHDuDsJwwAACAAQAAgAAAAIACAACAAAAAAAAAAAAiBgNHCPu3PMi2RSVEJl3Hn6FdhYoWxDTEdvVvg0I2V+/JKxzAuCxoMAAAgAEAAIAAAACAAgAAgAAAAAAAAAAAIgYDZu4oYzq12z164t18nKDxSXquyKRTdzIMfaa5Hc8E8egcXmtqBjAAAIABAACAAAAAgAIAAIAAAAAAAAAAACIGA/WJNsbQW4ZpqQc4wgJXx76N0J9BXtmtn8KS6CGnnVZPHE+CvLcwAACAAQAAgAAAAIACAACAAAAAAAAAAAAAIgICXD7c7JLk/bC0MJD5zpgML2uL4wMMEFXORQ4sBQdVVm8cT4K8tzAAAIABAACAAAAAgAIAAIAAAAAAAQAAACICAxgGi4BP18ux5lOP52MQ9PRPfXgtCjPccC5HIDXRIy2qHF5ragYwAACAAQAAgAAAAIACAACAAAAAAAEAAAAiAgN426KdAJSbNAOC2kc14X1IALYhIcCcdfxOl79acZT8uxw7g7CcMAAAgAEAAIAAAACAAgAAgAAAAAABAAAAIgIDx+fgEf5c9OQxAAZtBX8RDspq6IGsaeMfTHJwPx5cC1AcwLgsaDAAAIABAACAAAAAgAIAAIAAAAAAAQAAAAAiAgJS9efszeiZDbV1CzOv45rPRR1jh1L3pHTzV9blwWUyLxw7g7CcMAAAgAEAAIAAAACAAgAAgAEAAAAAAAAAIgIClDkC1BrgHuuy1WcCHkOQH4KEKYZuo9swFPDX2G+euh0cT4K8tzAAAIABAACAAAAAgAIAAIABAAAAAAAAACICArDjjAJxyeaP5PDwo79XG7P9MN1J9lcgmDMxcCuVMujqHF5ragYwAACAAQAAgAAAAIACAACAAQAAAAAAAAAiAgNrInUs30ukm4ERBVHjsJQfGbYb1dnVy50tk/ym/hyUQBzAuCxoMAAAgAEAAIAAAACAAgAAgAEAAAAAAAAAAA==";
        let search_radius = 5;
        let cosigners = wallet.get_signers(psbt, search_radius).unwrap();
        assert_eq!(cosigners.len(), 0);
    }

    #[test]
    fn test_wallet_get_signers_1_signer() {
        let multisig = get_test_multisig();
        let blockchain = get_blockchain();
        let wallet = Wallet::from_multisig(&blockchain, &multisig).unwrap();
        let psbt = "cHNidP8BAIkBAAAAAfx15Ttmz6elm9LHqX2jVvqboFTMUrD3OVilRE0RH3HNAQAAAAD9////AhAnAAAAAAAAIgAgapL4iNK+iOvUjmi74v5KOdJq0+brS2MsQt8bZu/jvVy37QAAAAAAACIAIDGu4FBMXgV+irxy6Vz78NrpoH/ezv1eabyuP2wfZkIWAAAAAAABAH0CAAAAAT+oqsgQzz8UeaO8LJJbqVkfwWeYbirVFcC2brDjkKl6AQAAAAD+////AqsLKDAAAAAAFgAUhEkMOtrFy4DnVXpDGJ/qgMsGz5NAGQEAAAAAACIAIJr0KH+bUEFUEaEMV+hkqL0I7NJwuXLireNKjH/A6A7lVNgiAAEBK0AZAQAAAAAAIgAgmvQof5tQQVQRoQxX6GSovQjs0nC5cuKt40qMf8DoDuUiAgNHCPu3PMi2RSVEJl3Hn6FdhYoWxDTEdvVvg0I2V+/JK0cwRAIgQevv55jDdhvw6pJAhjXknVP4JzISWX6RxjqjG3ACYGUCIDGoyjaEXMPEgCDnFti6dhClIHcHXzZpR20XPwDgOzXFAQEFi1MhArnezPqp/4uC8h2tGFRVE3r3OhwQphISKoDOVVAAyWLmIQNHCPu3PMi2RSVEJl3Hn6FdhYoWxDTEdvVvg0I2V+/JKyEDZu4oYzq12z164t18nKDxSXquyKRTdzIMfaa5Hc8E8eghA/WJNsbQW4ZpqQc4wgJXx76N0J9BXtmtn8KS6CGnnVZPVK4iBgK53sz6qf+LgvIdrRhUVRN69zocEKYSEiqAzlVQAMli5hw7g7CcMAAAgAEAAIAAAACAAgAAgAAAAAAAAAAAIgYDRwj7tzzItkUlRCZdx5+hXYWKFsQ0xHb1b4NCNlfvySscwLgsaDAAAIABAACAAAAAgAIAAIAAAAAAAAAAACIGA2buKGM6tds9euLdfJyg8Ul6rsikU3cyDH2muR3PBPHoHF5ragYwAACAAQAAgAAAAIACAACAAAAAAAAAAAAiBgP1iTbG0FuGaakHOMICV8e+jdCfQV7ZrZ/Ckughp51WTxxPgry3MAAAgAEAAIAAAACAAgAAgAAAAAAAAAAAACICAlw+3OyS5P2wtDCQ+c6YDC9ri+MDDBBVzkUOLAUHVVZvHE+CvLcwAACAAQAAgAAAAIACAACAAAAAAAEAAAAiAgMYBouAT9fLseZTj+djEPT0T314LQoz3HAuRyA10SMtqhxea2oGMAAAgAEAAIAAAACAAgAAgAAAAAABAAAAIgIDeNuinQCUmzQDgtpHNeF9SAC2ISHAnHX8Tpe/WnGU/LscO4OwnDAAAIABAACAAAAAgAIAAIAAAAAAAQAAACICA8fn4BH+XPTkMQAGbQV/EQ7KauiBrGnjH0xycD8eXAtQHMC4LGgwAACAAQAAgAAAAIACAACAAAAAAAEAAAAAIgICUvXn7M3omQ21dQszr+Oaz0UdY4dS96R081fW5cFlMi8cO4OwnDAAAIABAACAAAAAgAIAAIABAAAAAAAAACICApQ5AtQa4B7rstVnAh5DkB+ChCmGbqPbMBTw19hvnrodHE+CvLcwAACAAQAAgAAAAIACAACAAQAAAAAAAAAiAgKw44wCccnmj+Tw8KO/Vxuz/TDdSfZXIJgzMXArlTLo6hxea2oGMAAAgAEAAIAAAACAAgAAgAEAAAAAAAAAIgIDayJ1LN9LpJuBEQVR47CUHxm2G9XZ1cudLZP8pv4clEAcwLgsaDAAAIABAACAAAAAgAIAAIABAAAAAAAAAAA=";
        let search_radius = 5;
        let cosigners = wallet.get_signers(psbt, search_radius).unwrap();
        assert_eq!(cosigners.len(), 1);
        assert_eq!(cosigners[0], Cosigner{
      xfp:Some("c0b82c68".to_string()),
      derivation_path: Some("m/48'/1'/0'/2'".to_string()),
      xpub: "Vpub5knpWjcHt8uQ7xUWM9mDRWpKst81n7zzmtr2LDaH3GPHkMoVw41L3bDDSded6xioVcg7L3ozoiwfCEKPCVFoiiKy9yqkV6nejso8Puy7Mvf".to_string(),
    });
    }

    #[test]
    fn test_wallet_get_signers_2_signers() {
        let multisig = get_test_multisig();
        let blockchain = get_blockchain();
        let wallet = Wallet::from_multisig(&blockchain, &multisig).unwrap();
        let psbt = "cHNidP8BAIkBAAAAAfx15Ttmz6elm9LHqX2jVvqboFTMUrD3OVilRE0RH3HNAQAAAAD9////AhAnAAAAAAAAIgAgapL4iNK+iOvUjmi74v5KOdJq0+brS2MsQt8bZu/jvVy37QAAAAAAACIAIDGu4FBMXgV+irxy6Vz78NrpoH/ezv1eabyuP2wfZkIWAAAAAAABAH0CAAAAAT+oqsgQzz8UeaO8LJJbqVkfwWeYbirVFcC2brDjkKl6AQAAAAD+////AqsLKDAAAAAAFgAUhEkMOtrFy4DnVXpDGJ/qgMsGz5NAGQEAAAAAACIAIJr0KH+bUEFUEaEMV+hkqL0I7NJwuXLireNKjH/A6A7lVNgiAAEBK0AZAQAAAAAAIgAgmvQof5tQQVQRoQxX6GSovQjs0nC5cuKt40qMf8DoDuUiAgNHCPu3PMi2RSVEJl3Hn6FdhYoWxDTEdvVvg0I2V+/JK0cwRAIgQevv55jDdhvw6pJAhjXknVP4JzISWX6RxjqjG3ACYGUCIDGoyjaEXMPEgCDnFti6dhClIHcHXzZpR20XPwDgOzXFASICA2buKGM6tds9euLdfJyg8Ul6rsikU3cyDH2muR3PBPHoRzBEAiB5X6ILUU7AG8PfY9V/Ql5uP/19BHPvDzqVmuueZvDM8gIgcxvLCwoBZQeaAIFzvwEluHbu50F+E2W6dol7uXH5Oo0BAQWLUyECud7M+qn/i4LyHa0YVFUTevc6HBCmEhIqgM5VUADJYuYhA0cI+7c8yLZFJUQmXcefoV2FihbENMR29W+DQjZX78krIQNm7ihjOrXbPXri3XycoPFJeq7IpFN3Mgx9prkdzwTx6CED9Yk2xtBbhmmpBzjCAlfHvo3Qn0Fe2a2fwpLoIaedVk9UriIGArnezPqp/4uC8h2tGFRVE3r3OhwQphISKoDOVVAAyWLmHDuDsJwwAACAAQAAgAAAAIACAACAAAAAAAAAAAAiBgNHCPu3PMi2RSVEJl3Hn6FdhYoWxDTEdvVvg0I2V+/JKxzAuCxoMAAAgAEAAIAAAACAAgAAgAAAAAAAAAAAIgYDZu4oYzq12z164t18nKDxSXquyKRTdzIMfaa5Hc8E8egcXmtqBjAAAIABAACAAAAAgAIAAIAAAAAAAAAAACIGA/WJNsbQW4ZpqQc4wgJXx76N0J9BXtmtn8KS6CGnnVZPHE+CvLcwAACAAQAAgAAAAIACAACAAAAAAAAAAAAAIgICXD7c7JLk/bC0MJD5zpgML2uL4wMMEFXORQ4sBQdVVm8cT4K8tzAAAIABAACAAAAAgAIAAIAAAAAAAQAAACICAxgGi4BP18ux5lOP52MQ9PRPfXgtCjPccC5HIDXRIy2qHF5ragYwAACAAQAAgAAAAIACAACAAAAAAAEAAAAiAgN426KdAJSbNAOC2kc14X1IALYhIcCcdfxOl79acZT8uxw7g7CcMAAAgAEAAIAAAACAAgAAgAAAAAABAAAAIgIDx+fgEf5c9OQxAAZtBX8RDspq6IGsaeMfTHJwPx5cC1AcwLgsaDAAAIABAACAAAAAgAIAAIAAAAAAAQAAAAAiAgJS9efszeiZDbV1CzOv45rPRR1jh1L3pHTzV9blwWUyLxw7g7CcMAAAgAEAAIAAAACAAgAAgAEAAAAAAAAAIgIClDkC1BrgHuuy1WcCHkOQH4KEKYZuo9swFPDX2G+euh0cT4K8tzAAAIABAACAAAAAgAIAAIABAAAAAAAAACICArDjjAJxyeaP5PDwo79XG7P9MN1J9lcgmDMxcCuVMujqHF5ragYwAACAAQAAgAAAAIACAACAAQAAAAAAAAAiAgNrInUs30ukm4ERBVHjsJQfGbYb1dnVy50tk/ym/hyUQBzAuCxoMAAAgAEAAIAAAACAAgAAgAEAAAAAAAAAAA==";
        let search_radius = 5;
        let cosigners = wallet.get_signers(psbt, search_radius).unwrap();
        assert_eq!(cosigners.len(), 2);
        assert_eq!(cosigners[0], Cosigner{
      xfp:Some("c0b82c68".to_string()),
      derivation_path: Some("m/48'/1'/0'/2'".to_string()),
      xpub: "Vpub5knpWjcHt8uQ7xUWM9mDRWpKst81n7zzmtr2LDaH3GPHkMoVw41L3bDDSded6xioVcg7L3ozoiwfCEKPCVFoiiKy9yqkV6nejso8Puy7Mvf".to_string(),
    });
        assert_eq!(cosigners[1], Cosigner{
      xfp:Some("5e6b6a06".to_string()),
      derivation_path: Some("m/48'/1'/0'/2'".to_string()),
      xpub: "Vpub5mU6P8gQgQbHZWUuLexLgEjRt1WnW1KJscQSuDG9W9HbCHgTimuRG4k6ykX52HYH1uqmp832QypyKwqHkc3gVmQWoZmbXWGrYXyPS2SqHJZ".to_string(),
    });
    }

    #[test]
    #[should_panic(expected = "no cosigner found with finger print")]
    fn test_wallet_get_signers_should_fail_for_signer_not_found_in_cosigners() {
        let multisig = get_test_multisig();
        let blockchain = get_blockchain();
        let wallet = Wallet::from_multisig(&blockchain, &multisig).unwrap();
        let psbt = "cHNidP8BAH0BAAAAATu0VKInpjK6eFMaQAFunA3A+97jPjpWNZMcnccgbYoKAAAAAAD9////AigjAAAAAAAAFgAU644owvunvN0djfQv0x5LcEiio6iKzwAAAAAAACIAINkzVKLuqXXoM2JecLoKX3wTeO6uaLlPJsbi5mwkVz0wAAAAAAABAP1lAQEAAAAAAQE15cLrKDCX1vMnOWvNFP8hDTHLFmJQ9i8AmgPKVo+7ygAAAAAA/f///wI79gAAAAAAACIAIFmq2KP+sPGRXMuV0a6osWkFyavpWxVY1ntadmhNcOusECcAAAAAAAAiACBrVZTHqoRBO0A6nn7Ao3t+megE/OELlG9ac49wzSjWNwQARzBEAiADdX44UYMWWxiTV4D4KTZBsnVVNyPoPaA54V0/y/Xi2AIgP6e+myoLB9IKIuOh/1UHis+7nUD/fHRf78aWFFACg1QBRzBEAiBA8AwwW71HvR4k9p3M/Jg4/9KxQUFDVaE/4vDN2z10SQIgAZQt6cZpSkd8VMRCBw3R6DwoWJPs97817oSiz+u6GdoBR1IhAlSfGcs3T+fKul2/pLHdc1bcPl5YMhVt/ju3qZZCvs7LIQPaX07glZnKOZPognhq6J7tD4amJrnC0zfX4MOTfsPse1KuAAAAACICAigcEWvhkrprJLufO8bCAvkqDMhUa8iTQ7aiwqT6ubc4RzBEAiBahqxx/ujI2XaIZth3P5n48he7kbrL7atIVgGmDE2IIgIgcxvUi29488IxIYvbB0z6xj1qa2YyL/TV4QAOMaLVpLcBAQVHUiECKBwRa+GSumsku587xsIC+SoMyFRryJNDtqLCpPq5tzghA5D7+vnb18WzTEh/VZ8RsWEmKYQXGZ+q2x/Afdz02vneUq4iBgIoHBFr4ZK6ayS7nzvGwgL5KgzIVGvIk0O2osKk+rm3OBD3w4rRAQAAgAEAAAABAAAAIgYDkPv6+dvXxbNMSH9VnxGxYSYphBcZn6rbH8B93PTa+d4Q6vi1bwEAAIABAAAAAQAAAAAAAQFHUiECQiTXOO7VU+8GV6lQWlRwidB4tapVE/cqCW36G0PgRHwhA9INPKlYclyE+qE4tNQmf/Jw00Z3rf79NCU4xaIxe8GaUq4iAgJCJNc47tVT7wZXqVBaVHCJ0Hi1qlUT9yoJbfobQ+BEfBD3w4rRAQAAgAEAAAAEAAAAIgID0g08qVhyXIT6oTi01CZ/8nDTRnet/v00JTjFojF7wZoQ6vi1bwEAAIABAAAABAAAAAA=";
        let search_radius = 5;
        wallet.get_signers(psbt, search_radius).unwrap();
    }

    fn get_test_multisig() -> Multisig {
        let cosigner1 = Cosigner{
      xfp:Some("c0b82c68".to_string()),
      derivation_path: Some("m/48'/1'/0'/2'".to_string()),
      xpub: "Vpub5knpWjcHt8uQ7xUWM9mDRWpKst81n7zzmtr2LDaH3GPHkMoVw41L3bDDSded6xioVcg7L3ozoiwfCEKPCVFoiiKy9yqkV6nejso8Puy7Mvf".to_string(),
    };

        let cosigner2 = Cosigner{
      xfp:Some("5e6b6a06".to_string()),
      derivation_path: Some("m/48'/1'/0'/2'".to_string()),
      xpub: "Vpub5mU6P8gQgQbHZWUuLexLgEjRt1WnW1KJscQSuDG9W9HbCHgTimuRG4k6ykX52HYH1uqmp832QypyKwqHkc3gVmQWoZmbXWGrYXyPS2SqHJZ".to_string(),
    };

        let cosigner3 = Cosigner{
      xfp:Some("4f82bcb7".to_string()),
      derivation_path: Some("m/48'/1'/0'/2'".to_string()),
      xpub: "Vpub5kyc22JJ36vUUs5LJzSNAqToEYkDhxXHnCK4n7EsYHv5QCcZkyNGL8f3k8pLdgJGT7iboaTwf6mfhmnkCpqF5YZQisNsbGmemzPGzMZCvbC".to_string(),
    };

        let cosigner4 = Cosigner{
      xfp:Some("3b83b09c".to_string()),
      derivation_path: Some("m/48'/1'/0'/2'".to_string()),
      xpub: "Vpub5nNXHiWYY8Q19GXv7bpcvHEbDbRQui6H3kNSdFoimEZNtcT7uK7MSqpS4MdHF5BJATzCeyjrRdy4asZXghd4VgRoWo4kyuf4k6cUhtVrKQV".to_string(),
    };
        let mut multisig = Multisig::new(3);
        multisig.add_cosigner(cosigner1);
        multisig.add_cosigner(cosigner2);
        multisig.add_cosigner(cosigner3);
        multisig.add_cosigner(cosigner4);

        multisig
    }

    fn get_blockchain() -> Blockchain {
        Blockchain::new(
            "ssl://electrum.blockstream.info:60002",
            bitcoin::Network::Testnet,
        )
        .unwrap()
    }
}
