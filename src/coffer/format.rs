use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use zeroize::Zeroizing;

use super::{error::KeyTideError, key::SecretKey};

const CONTAINER_MAGIC: &[u8; 8] = b"COFFER\0\x01";
const VERSION: u8 = 1;
const ALGORITHM_AES_256_GCM: u8 = 1;
const PREFIX_LEN: usize = 30;
const TAG_LEN: usize = 16;
const MAX_FILENAME_LEN: usize = 1024;

pub(super) struct DecryptedPayload {
    pub filename: String,
    pub bytes: Zeroizing<Vec<u8>>,
}

pub(super) fn encrypt(
    filename: &str,
    plaintext: &[u8],
    key: &SecretKey,
) -> Result<Vec<u8>, KeyTideError> {
    let mut nonce = [0_u8; 12];
    getrandom::fill(&mut nonce).map_err(|_| KeyTideError::RandomFailed)?;
    encrypt_with_nonce(filename, plaintext, key, nonce)
}

fn encrypt_with_nonce(
    filename: &str,
    plaintext: &[u8],
    key: &SecretKey,
    nonce: [u8; 12],
) -> Result<Vec<u8>, KeyTideError> {
    validate_filename(filename)?;
    let filename_bytes = filename.as_bytes();
    let filename_len =
        u16::try_from(filename_bytes.len()).map_err(|_| KeyTideError::InvalidFilename)?;
    let plaintext_len = u64::try_from(plaintext.len()).map_err(|_| KeyTideError::FileTooLarge)?;

    let payload_capacity = 2_usize
        .checked_add(filename_bytes.len())
        .and_then(|value| value.checked_add(8))
        .and_then(|value| value.checked_add(plaintext.len()))
        .ok_or(KeyTideError::FileTooLarge)?;
    let mut encoded_payload = Zeroizing::new(Vec::with_capacity(payload_capacity));
    encoded_payload.extend_from_slice(&filename_len.to_be_bytes());
    encoded_payload.extend_from_slice(filename_bytes);
    encoded_payload.extend_from_slice(&plaintext_len.to_be_bytes());
    encoded_payload.extend_from_slice(plaintext);

    let ciphertext_len = encoded_payload
        .len()
        .checked_add(TAG_LEN)
        .ok_or(KeyTideError::FileTooLarge)?;
    let ciphertext_len_u64 =
        u64::try_from(ciphertext_len).map_err(|_| KeyTideError::FileTooLarge)?;
    let prefix = encode_prefix(nonce, ciphertext_len_u64);
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes()).map_err(|_| KeyTideError::InvalidKey)?;
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &encoded_payload,
                aad: &prefix,
            },
        )
        .map_err(|_| KeyTideError::AuthenticationFailed)?;

    let mut container = Vec::with_capacity(PREFIX_LEN + ciphertext.len());
    container.extend_from_slice(&prefix);
    container.extend_from_slice(&ciphertext);
    Ok(container)
}

pub(super) fn decrypt(container: &[u8], key: &SecretKey) -> Result<DecryptedPayload, KeyTideError> {
    let (prefix, ciphertext) = parse_container(container)?;
    let nonce: &[u8; 12] = prefix[10..22]
        .try_into()
        .map_err(|_| KeyTideError::InvalidContainer)?;
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes()).map_err(|_| KeyTideError::InvalidKey)?;
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad: prefix,
            },
        )
        .map_err(|_| KeyTideError::AuthenticationFailed)?;
    parse_payload(Zeroizing::new(plaintext))
}

fn encode_prefix(nonce: [u8; 12], ciphertext_len: u64) -> [u8; PREFIX_LEN] {
    let mut prefix = [0_u8; PREFIX_LEN];
    prefix[..8].copy_from_slice(CONTAINER_MAGIC);
    prefix[8] = VERSION;
    prefix[9] = ALGORITHM_AES_256_GCM;
    prefix[10..22].copy_from_slice(&nonce);
    prefix[22..30].copy_from_slice(&ciphertext_len.to_be_bytes());
    prefix
}

fn parse_container(container: &[u8]) -> Result<(&[u8], &[u8]), KeyTideError> {
    if container.len() < PREFIX_LEN || container.get(..8) != Some(CONTAINER_MAGIC) {
        return Err(KeyTideError::InvalidContainer);
    }
    if container[8] != VERSION {
        return Err(KeyTideError::UnsupportedVersion(container[8]));
    }
    if container[9] != ALGORITHM_AES_256_GCM {
        return Err(KeyTideError::UnsupportedAlgorithm(container[9]));
    }
    let declared = u64::from_be_bytes(
        container[22..30]
            .try_into()
            .map_err(|_| KeyTideError::InvalidContainer)?,
    );
    let declared = usize::try_from(declared).map_err(|_| KeyTideError::FileTooLarge)?;
    if declared < TAG_LEN || container.len().checked_sub(PREFIX_LEN) != Some(declared) {
        return Err(KeyTideError::InvalidContainer);
    }
    Ok(container.split_at(PREFIX_LEN))
}

fn parse_payload(payload: Zeroizing<Vec<u8>>) -> Result<DecryptedPayload, KeyTideError> {
    let filename_len_bytes = payload.get(..2).ok_or(KeyTideError::InvalidContainer)?;
    let filename_len = usize::from(u16::from_be_bytes(
        filename_len_bytes
            .try_into()
            .map_err(|_| KeyTideError::InvalidContainer)?,
    ));
    if filename_len == 0 || filename_len > MAX_FILENAME_LEN {
        return Err(KeyTideError::InvalidFilename);
    }
    let name_end = 2_usize
        .checked_add(filename_len)
        .ok_or(KeyTideError::InvalidContainer)?;
    let size_end = name_end
        .checked_add(8)
        .ok_or(KeyTideError::InvalidContainer)?;
    let filename_bytes = payload
        .get(2..name_end)
        .ok_or(KeyTideError::InvalidContainer)?;
    let filename = std::str::from_utf8(filename_bytes)
        .map_err(|_| KeyTideError::InvalidFilename)?
        .to_owned();
    validate_filename(&filename)?;
    let declared_size = u64::from_be_bytes(
        payload
            .get(name_end..size_end)
            .ok_or(KeyTideError::InvalidContainer)?
            .try_into()
            .map_err(|_| KeyTideError::InvalidContainer)?,
    );
    let bytes = payload
        .get(size_end..)
        .ok_or(KeyTideError::InvalidContainer)?;
    if u64::try_from(bytes.len()).map_err(|_| KeyTideError::FileTooLarge)? != declared_size {
        return Err(KeyTideError::InvalidContainer);
    }
    Ok(DecryptedPayload {
        filename,
        bytes: Zeroizing::new(bytes.to_vec()),
    })
}

pub(super) fn validate_filename(filename: &str) -> Result<(), KeyTideError> {
    let bytes = filename.as_bytes();
    if bytes.is_empty()
        || bytes.len() > MAX_FILENAME_LEN
        || filename == "."
        || filename == ".."
        || filename.contains(['\0', '/', '\\'])
        || std::path::Path::new(filename).components().count() != 1
    {
        return Err(KeyTideError::InvalidFilename);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (SecretKey, Vec<u8>) {
        let key = SecretKey([0x11; 32]);
        let container = encrypt_with_nonce("note.txt", b"hello", &key, [0x22; 12]).unwrap();
        (key, container)
    }

    #[test]
    fn deterministic_fixture_is_byte_for_byte_stable() {
        let (_, container) = fixture();
        let encoded = container
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(
            encoded,
            "434f4646455200010101222222222222222222222222000000000000002717ff6926b4aab12b9d4bde3c48a6e9d51f96a58706edc3edae22ac447457f57b1a54dfb50676d9"
        );
    }

    #[test]
    fn payload_round_trips_binary_content() {
        let key = SecretKey([9; 32]);
        let container = encrypt_with_nonce("data.bin", &[0, 1, 0xff, 4], &key, [3; 12]).unwrap();
        let result = decrypt(&container, &key).unwrap();
        assert_eq!(result.filename, "data.bin");
        assert_eq!(&*result.bytes, &[0, 1, 0xff, 4]);
    }

    #[test]
    fn wrong_key_and_authenticated_changes_share_one_error() {
        let (key, container) = fixture();
        assert!(matches!(
            decrypt(&container, &SecretKey([0x33; 32])),
            Err(KeyTideError::AuthenticationFailed)
        ));
        for index in [10, 21, 30, container.len() - 1] {
            let mut changed = container.clone();
            changed[index] ^= 1;
            assert!(matches!(
                decrypt(&changed, &key),
                Err(KeyTideError::AuthenticationFailed)
            ));
        }
    }

    #[test]
    fn container_rejects_truncation_trailing_data_and_unknown_fields() {
        let (key, container) = fixture();
        for length in [0, 29, container.len() - 1] {
            assert!(decrypt(&container[..length], &key).is_err());
        }
        let mut trailing = container.clone();
        trailing.push(0);
        assert!(matches!(
            decrypt(&trailing, &key),
            Err(KeyTideError::InvalidContainer)
        ));
        let mut version = container.clone();
        version[8] = 2;
        assert!(matches!(
            decrypt(&version, &key),
            Err(KeyTideError::UnsupportedVersion(2))
        ));
        let mut algorithm = container;
        algorithm[9] = 2;
        assert!(matches!(
            decrypt(&algorithm, &key),
            Err(KeyTideError::UnsupportedAlgorithm(2))
        ));
    }

    #[test]
    fn filenames_are_strictly_safe() {
        for invalid in ["", ".", "..", "../x", "a/b", "a\\b", "nul\0x"] {
            assert!(matches!(
                validate_filename(invalid),
                Err(KeyTideError::InvalidFilename)
            ));
        }
        assert!(validate_filename("résumé.txt").is_ok());
    }

    #[test]
    fn payload_parser_rejects_invalid_utf8_truncation_and_size_mismatch() {
        for payload in [vec![], vec![0], vec![0, 1], vec![0, 1, b'a']] {
            assert!(parse_payload(Zeroizing::new(payload)).is_err());
        }

        let mut invalid_utf8 = vec![0, 1, 0xff];
        invalid_utf8.extend_from_slice(&0_u64.to_be_bytes());
        assert!(matches!(
            parse_payload(Zeroizing::new(invalid_utf8)),
            Err(KeyTideError::InvalidFilename)
        ));

        let mut wrong_size = vec![0, 1, b'a'];
        wrong_size.extend_from_slice(&2_u64.to_be_bytes());
        wrong_size.push(1);
        assert!(matches!(
            parse_payload(Zeroizing::new(wrong_size)),
            Err(KeyTideError::InvalidContainer)
        ));
    }
}
