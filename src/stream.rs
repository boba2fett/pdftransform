use std::{task::{Context, Poll}, pin::Pin};

use futures::{ready};
use tokio::{io::{self, ReadBuf}};

use futures::stream::Stream;

pub struct StreamReader<S> {
    pub stream: S,
}

impl<S> io::AsyncRead for StreamReader<S>
where
    S: Stream<Item = Vec<u8>> + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let stream = Pin::new(&mut self.stream);
        let chunk = match ready!(stream.poll_next(cx)) {
            Some(chunk) => chunk,
            None => return Poll::Ready(Ok(())),
        };

        let len = buf.remaining().min(chunk.len());
        buf.put_slice(&chunk[..len]);
        
        Poll::Ready(Ok(()))
    }
}