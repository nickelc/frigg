use reqwest::header::HeaderValue;

use crate::Error;

#[derive(Debug)]
pub struct Nonce {
    pub encoded: String,
    pub value: String,
    pub signature: String,
}

impl TryFrom<&HeaderValue> for Nonce {
    type Error = Error;

    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        let encoded = value.to_str()?.to_owned();
        let nonce = decrypt_nonce(encoded.as_bytes())?;
        let signature = gen_sig(nonce.as_bytes())?;

        Ok(Nonce {
            encoded,
            value: nonce,
            signature,
        })
    }
}

#[derive(Debug)]
pub struct Session {
    pub nonce: Nonce,
    pub id: String,
}

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use base64ct::{Base64, Encoding};

type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;

const KEY_1: &[u8] = b"vicopx7dqu06emacgpnpy8j8zwhduwlh";
const KEY_2: &[u8] = b"9u7qab84rpc16gvk";

pub fn calc_logic_check(input: &str, nonce: &str) -> String {
    nonce
        .chars()
        .flat_map(|c| {
            let n = c as usize & 0xf;
            input.get(n..n + 1)
        })
        .collect()
}

fn decrypt_nonce(nonce: &[u8]) -> Result<String, Error> {
    let mut buf = vec![0; 32];
    Base64::decode(nonce, &mut buf)?;

    let cipher = Aes256CbcDec::new_from_slices(KEY_1, &KEY_1[..16])?;
    let buf = cipher.decrypt_padded_mut::<Pkcs7>(&mut buf)?;
    Ok(String::from_utf8(buf.to_vec())?)
}

fn gen_sig(nonce: &[u8]) -> Result<String, Error> {
    let derived_key: Vec<_> = nonce[0..16]
        .iter()
        .map(|c| KEY_1[(c % 16) as usize])
        .chain(KEY_2.iter().copied())
        .collect();

    let cipher = Aes256CbcEnc::new_from_slices(&derived_key, &derived_key[..16])?;
    let mut data = [0; 44];
    let pos = nonce.len();
    data[..pos].copy_from_slice(nonce);
    let data = cipher.encrypt_padded_mut::<Pkcs7>(&mut data, pos)?;

    let mut sig = vec![0; Base64::encoded_len(data)];
    Base64::encode(data, &mut sig)?;
    Ok(String::from_utf8(sig)?)
}
