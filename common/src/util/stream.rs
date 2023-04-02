use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::ready;
use tokio::io::{self, ReadBuf};

use futures::stream::Stream;

pub struct StreamReader<S> {
    pub stream: S,
    pub buffer: Vec<u8>,
}

impl<S> io::AsyncRead for StreamReader<S>
where
    S: Stream<Item = Vec<u8>> + Unpin,
{
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        let outstanding = buf.remaining().min(self.buffer.len());
        if outstanding > 0 {
            buf.put_slice(&self.buffer[..outstanding]);
            self.buffer.drain(..outstanding);
            return Poll::Ready(Ok(()));
        }
        let stream = Pin::new(&mut self.stream);
        let chunk = match ready!(stream.poll_next(cx)) {
            Some(chunk) => chunk,
            None => return Poll::Ready(Ok(())),
        };

        let len = buf.remaining().min(chunk.len());
        buf.put_slice(&chunk[..len]);
        self.buffer = chunk[len..].to_vec();

        Poll::Ready(Ok(()))
    }
}
