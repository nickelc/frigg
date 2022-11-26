use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

use indicatif::{ProgressBar, ProgressStyle};
use tokio::io::{AsyncRead, ReadBuf};

pub fn new(size: u64) -> ProgressBar {
    let style = ProgressStyle::default_bar()
        .template("{spinner:.green} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes_per_sec} {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-");
    let pb = ProgressBar::new(size);
    pb.set_style(style);

    pb
}

pub fn wrap_reader<T>(inner: T, pb: ProgressBar) -> AsyncProgress<T>
where
    T: AsyncRead + Unpin,
{
    AsyncProgress { inner, pb }
}

pub struct AsyncProgress<T> {
    inner: T,
    pb: ProgressBar,
}

impl<T> AsyncRead for AsyncProgress<T>
where
    T: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), Error>> {
        let result = Pin::new(&mut self.inner).poll_read(cx, buf);
        self.pb.inc(buf.filled().len() as u64);
        result
    }
}
