use std::fmt;
use std::io;
use std::pin::Pin;

use futures_util::future::poll_fn;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};

use aes::cipher::block_padding::{Pkcs7, UnpadError};
use aes::cipher::generic_array::{ArrayLength, GenericArray};
use aes::cipher::{BlockDecrypt, InvalidLength, KeyInit};
use aes::Aes128;

const BUF_SIZE: usize = 4128;
const BLOCK_SIZE: usize = 4096;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidKey,
    Unpad,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(_) => f.write_str("io error"),
            Self::InvalidKey => f.write_str("invalid key"),
            Self::Unpad => f.write_str("unpadding error"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<InvalidLength> for Error {
    fn from(_: InvalidLength) -> Self {
        Self::InvalidKey
    }
}

impl From<UnpadError> for Error {
    fn from(_: UnpadError) -> Self {
        Self::Unpad
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

    let cipher = Aes128::new_from_slice(key)?;
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
                let (block, remainder) = buf.filled_mut().split_at_mut(BLOCK_SIZE);
                cipher.decrypt_blocks(to_blocks(block));
                writer.write_all(block).await?;
                amt += block.len() as u64;
                remainder.len()
            };
            buf.filled_mut().copy_within(BLOCK_SIZE.., 0);
            buf.set_filled(new_filled);
        } else {
            let block = buf.filled_mut();
            let buf = cipher.decrypt_padded::<Pkcs7>(block)?;

            writer.write_all(buf).await?;
            amt += buf.len() as u64;
        }
    }
    writer.flush().await?;
    Ok(amt)
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
