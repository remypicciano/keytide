use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use super::error::KeyTideError;

pub(super) const KEY_MAGIC: &[u8; 8] = b"COFKEY\0\x01";
pub(super) const KEY_FILE_LEN: usize = 44;
const VERSION: u8 = 1;
const ALGORITHM_AES_256_GCM: u8 = 1;

#[derive(Zeroize, ZeroizeOnDrop)]
pub(super) struct SecretKey(pub(super) [u8; 32]);

impl SecretKey {
    pub(super) fn generate() -> Result<Self, KeyTideError> {
        let mut bytes = [0_u8; 32];
        getrandom::fill(&mut bytes).map_err(|_| KeyTideError::RandomFailed)?;
        Ok(Self(bytes))
    }

    pub(super) fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

pub(super) fn encode(key: &SecretKey) -> Zeroizing<Vec<u8>> {
    let mut encoded = Zeroizing::new(Vec::with_capacity(KEY_FILE_LEN));
    encoded.extend_from_slice(KEY_MAGIC);
    encoded.push(VERSION);
    encoded.push(ALGORITHM_AES_256_GCM);
    encoded.extend_from_slice(&[0, 0]);
    encoded.extend_from_slice(key.as_bytes());
    encoded
}

pub(super) fn parse(bytes: &[u8]) -> Result<SecretKey, KeyTideError> {
    if bytes.len() != KEY_FILE_LEN
        || bytes.get(..8) != Some(KEY_MAGIC)
        || bytes.get(10..12) != Some(&[0, 0])
    {
        return Err(KeyTideError::InvalidKey);
    }
    if bytes[8] != VERSION || bytes[9] != ALGORITHM_AES_256_GCM {
        return Err(KeyTideError::InvalidKey);
    }
    let material: [u8; 32] = bytes[12..44]
        .try_into()
        .map_err(|_| KeyTideError::InvalidKey)?;
    Ok(SecretKey(material))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_file_is_exactly_44_bytes_and_round_trips() {
        let key = SecretKey([7; 32]);
        let encoded = encode(&key);
        assert_eq!(encoded.len(), KEY_FILE_LEN);
        assert_eq!(parse(&encoded).unwrap().as_bytes(), key.as_bytes());
    }

    #[test]
    fn key_parser_rejects_every_structural_deviation() {
        let key = SecretKey([7; 32]);
        let valid = encode(&key);
        for length in [0, 43, 45] {
            assert!(matches!(
                parse(&vec![0; length]),
                Err(KeyTideError::InvalidKey)
            ));
        }
        for index in 0..12 {
            let mut changed = valid.to_vec();
            changed[index] ^= 1;
            assert!(matches!(parse(&changed), Err(KeyTideError::InvalidKey)));
        }
    }
}
