use xyzpub;
use crate::hbdk::errors::Error;

const TESTNET_XPUB_MULTISIG_PREFIXES: [&str;2] = ["Upub", "Vpub"];
const TESTNET_XPUB_PREFIXES: [&str;5] = ["tpub", "upub", "vpub", "Upub", "Vpub"];
const MAINNET_XPUB_MULTISIG_PREFIXES: [&str;2] = ["Ypub", "Zpub"];
const MAINNET_XPUB_PREFIXES: [&str;5] = ["xpub", "ypub", "zpub", "Ypub", "Zpub"];

pub fn to_legacy_xpub(xpub: &str)-> Result<String, Error>{
  Ok(xyzpub::convert_version(xpub, &get_legacy_version(xpub)?)?)
}

pub fn to_segwit_native_multisig_xpub(xpub: &str)-> Result<String, Error>{
  Ok(xyzpub::convert_version(xpub, &get_segwit_native_multisig_version(xpub)?)?)
}

pub fn get_segwit_native_multisig_version(xpub: &str) -> Result<xyzpub::Version, Error> {
  if is_testnet_xpub(xpub) {
    Ok(xyzpub::Version::VpubMultisig)
  } else if is_mainnet_xpub(xpub) {
    Ok(xyzpub::Version::ZpubMultisig)
  } else {
    Err(Error::new("Unknown xpub version"))
  }
}

pub fn get_legacy_version(xpub: &str) -> Result<xyzpub::Version, Error> {
  if is_testnet_xpub(xpub) {
    Ok(xyzpub::Version::Tpub)
  } else if is_mainnet_xpub(xpub) {
    Ok(xyzpub::Version::Xpub)
  } else {
    Err(Error::new("Unknown xpub version"))
  }
}

pub fn is_testnet_xpub(xpub: &str) -> bool {
  has_prefix(xpub, &TESTNET_XPUB_PREFIXES)
}

pub fn is_mainnet_xpub(xpub: &str) -> bool {
  has_prefix(xpub, &MAINNET_XPUB_PREFIXES)
}

pub fn is_testnet_multisig_xpub(xpub: &str) -> bool {
  has_prefix(xpub, &TESTNET_XPUB_MULTISIG_PREFIXES)
}

pub fn is_mainnet_multisig_xpub(xpub: &str) -> bool {
  has_prefix(xpub, &MAINNET_XPUB_MULTISIG_PREFIXES)
}

pub fn is_multisig_xpub(xpub: &str) -> bool {
  return is_mainnet_multisig_xpub(xpub) || is_testnet_multisig_xpub(xpub)
}

fn has_prefix(value: &str, prefixes: &[&str]) -> bool {
  for prefix in prefixes {
    if value.starts_with(prefix) {
      return true
    }
  }
  return false
}


#[cfg(test)]
mod tests {

  use super::*;
  #[test]
  fn test_is_testnet_xpub() {
    assert_eq!(is_testnet_xpub("vpub5UJtN2FGcxFk32jUXSSFV2keY3qNYWawtiHgNJLVnNTtYPpT47SvMm7Q9MiBQHVP5VE9rRask1mZzMDmw8f6XZhSwMx85TNNAyJwwqHDbgy"), true);
    assert_eq!(is_testnet_xpub("zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi"), false);
  }

  #[test]
  fn test_is_mainnet_xpub() {
    assert_eq!(is_mainnet_xpub("Vpub5fCyVFyiBup7VKTCTX1vrMP4h2rjz8mxkmwzS8PZ1hZVf3U1AhKU49AYkom3KXDS4jLNyvnvobWkpkESVT3n8RkpwKWCBcUiV3y7wFFRktE"), false);
    assert_eq!(is_mainnet_xpub("Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi"), true);
  }

  #[test]
  fn test_is_testnet_multisig_xpub() {
    assert_eq!(is_testnet_multisig_xpub("Vpub5fCyVFyiBup7VKTCTX1vrMP4h2rjz8mxkmwzS8PZ1hZVf3U1AhKU49AYkom3KXDS4jLNyvnvobWkpkESVT3n8RkpwKWCBcUiV3y7wFFRktE"), true);
    assert_eq!(is_testnet_multisig_xpub("vpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi"), false);
    assert_eq!(is_testnet_multisig_xpub("Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi"), false);
  }

  #[test]
  fn test_is_mainnet_multisig_xpub() {
    assert_eq!(is_mainnet_multisig_xpub("Vpub5fCyVFyiBup7VKTCTX1vrMP4h2rjz8mxkmwzS8PZ1hZVf3U1AhKU49AYkom3KXDS4jLNyvnvobWkpkESVT3n8RkpwKWCBcUiV3y7wFFRktE"), false);
    assert_eq!(is_mainnet_multisig_xpub("zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi"), false);
    assert_eq!(is_mainnet_multisig_xpub("Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi"), true);
  }

  #[test]
  fn test_is_multisig_xpub() {
    assert_eq!(is_multisig_xpub("Vpub5fCyVFyiBup7VKTCTX1vrMP4h2rjz8mxkmwzS8PZ1hZVf3U1AhKU49AYkom3KXDS4jLNyvnvobWkpkESVT3n8RkpwKWCBcUiV3y7wFFRktE"), true);
    assert_eq!(is_multisig_xpub("zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi"), false);
    assert_eq!(is_multisig_xpub("Zpub75bKLk9fCjgfELzLr2XS5TEcCXXGrci4EDwAcppFNBDwpNy53JhJS8cbRjdv39noPDKSfzK7EPC1Ciyfb7jRwY7DmiuYJ6WDr2nEL6yTkHi"), true);
  }
}