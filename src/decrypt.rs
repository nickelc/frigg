use std::fmt;
use std::io;
use std::pin::Pin;

use futures_util::future::poll_fn;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};

use aes::cipher::generic_array::{ArrayLength, GenericArray};
use aes::{Aes128, BlockCipher, NewBlockCipher};
use block_modes::block_padding::{Padding, Pkcs7, UnpadError};
use block_modes::{BlockMode, Ecb, InvalidKeyIvLength};

const BUF_SIZE: usize = 4128;
const BLOCK_SIZE: usize = 4096;

type Aes128Ecb = Ecb<Aes128, Pkcs7>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidKey,
    UnpadError,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => f.write_str("io error"),
            Self::InvalidKey => f.write_str("invalid key"),
            Self::UnpadError => f.write_str("unpadding error"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<InvalidKeyIvLength> for Error {
    fn from(_: InvalidKeyIvLength) -> Self {
        Self::InvalidKey
    }
}

impl From<UnpadError> for Error {
    fn from(_: UnpadError) -> Self {
        Self::UnpadError
    }
}

pub async fn decrypt<'a, R, W>(
    key: &[u8],
    mut reader: &'a mut R,
    writer: &'a mut W,
) -> Result<u64, Error>
where
    R: AsyncRead + Unpin + ?Sized,
    W: AsyncWrite + Unpin + ?Sized,
{
    let mut buf = vec![0; BUF_SIZE];
    let mut buf = ReadBuf::new(&mut buf);

    let mut cipher = Aes128Ecb::new_from_slices(&key, Default::default())?;
    let mut eof = false;
    let mut amt = 0;

    while !eof {
        while buf.remaining() > 0 {
            let rem = buf.remaining();
            poll_fn(|cx| Pin::new(&mut reader).poll_read(cx, &mut buf)).await?;
            if buf.remaining() == rem {
                eof = true;
                break;
            }
        }

        if buf.filled().len() > BLOCK_SIZE {
            let new_filled = {
                let (mut block, remainder) = buf.filled_mut().split_at_mut(BLOCK_SIZE);
                decrypt_blocks(&mut cipher, &mut block);
                writer.write_all(&block).await?;
                amt += block.len() as u64;
                remainder.len()
            };
            buf.filled_mut().copy_within(BLOCK_SIZE.., 0);
            buf.set_filled(new_filled);
        } else {
            decrypt_blocks(&mut cipher, buf.filled_mut());

            let buf = Pkcs7::unpad(buf.filled())?;
            writer.write_all(buf).await?;
            amt += buf.len() as u64;
        }
    }
    writer.flush().await?;
    Ok(amt)
}

fn decrypt_blocks<M, C>(cipher: &mut M, mut ciphertext_blocks: &mut [u8])
where
    M: BlockMode<C, Pkcs7>,
    C: BlockCipher + NewBlockCipher,
{
    cipher.decrypt_blocks(to_blocks(&mut ciphertext_blocks));
}

fn to_blocks<N>(data: &mut [u8]) -> &mut [GenericArray<u8, N>]
where
    N: ArrayLength<u8>,
{
    let n = N::to_usize();
    debug_assert!(data.len() % n == 0);

    #[allow(unsafe_code)]
    unsafe {
        std::slice::from_raw_parts_mut(data.as_ptr() as *mut GenericArray<u8, N>, data.len() / n)
    }
}
