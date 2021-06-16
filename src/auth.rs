use std::convert::TryFrom;

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
        Self::try_from(value.as_bytes())
    }
}

impl TryFrom<&[u8]> for Nonce {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let encoded = String::from_utf8(value.to_vec())?;
        let nonce = decrypt_nonce(value)?;
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

use aes::Aes256;
use base64ct::{Base64, Encoding};
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

const KEY_1: &[u8] = b"hqzdurufm2c8mf6bsjezu1qgveouv7c7";
const KEY_2: &[u8] = b"w13r4cvf4hctaujv";

pub fn calc_logic_check(input: &str, nonce: &str) -> String {
    nonce
        .chars()
        .flat_map(|c| {
            let n = c as usize & 0xf;
            input.get(n..n + 1)
        })
        .collect()
}

fn decrypt_nonce(nonce: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let mut buf = vec![0; 32];
    Base64::decode(nonce, &mut buf)?;

    let cipher = Aes256Cbc::new_from_slices(KEY_1, &KEY_1[..16])?;
    let len = cipher.decrypt(&mut buf)?.len();
    buf.truncate(len);
    Ok(String::from_utf8(buf)?)
}

fn gen_sig(nonce: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let derived_key: Vec<_> = nonce[0..16]
        .iter()
        .map(|c| KEY_1[(c % 16) as usize])
        .chain(KEY_2.iter().copied())
        .collect();

    let cipher = Aes256Cbc::new_from_slices(&derived_key, &derived_key[..16])?;
    let mut data = [0; 44];
    let pos = nonce.len();
    data[..pos].copy_from_slice(nonce);
    let data = cipher.encrypt(&mut data, pos)?;

    let mut sig = vec![0; Base64::encoded_len(&data)];
    Base64::encode(&data, &mut sig)?;
    Ok(String::from_utf8(sig)?)
}
