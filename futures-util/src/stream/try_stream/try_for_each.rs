use core::fmt;
use core::pin::Pin;
use futures_core::future::{Future, TryFuture};
use futures_core::stream::TryStream;
use futures_core::task::{Context, Poll};
use pin_project::{pin_project, project};

/// Future for the [`try_for_each`](super::TryStreamExt::try_for_each) method.
#[pin_project]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct TryForEach<St, Fut, F> {
    #[pin]
    stream: St,
    f: F,
    #[pin]
    future: Option<Fut>,
}

impl<St, Fut, F> fmt::Debug for TryForEach<St, Fut, F>
where
    St: fmt::Debug,
    Fut: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TryForEach")
            .field("stream", &self.stream)
            .field("future", &self.future)
            .finish()
    }
}

impl<St, Fut, F> TryForEach<St, Fut, F>
where St: TryStream,
      F: FnMut(St::Ok) -> Fut,
      Fut: TryFuture<Ok = (), Error = St::Error>,
{
    pub(super) fn new(stream: St, f: F) -> TryForEach<St, Fut, F> {
        TryForEach {
            stream,
            f,
            future: None,
        }
    }
}

impl<St, Fut, F> Future for TryForEach<St, Fut, F>
    where St: TryStream,
          F: FnMut(St::Ok) -> Fut,
          Fut: TryFuture<Ok = (), Error = St::Error>,
{
    type Output = Result<(), St::Error>;

    #[project]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        #[project]
        let TryForEach { mut stream, f, mut future } = self.project();
        loop {
            if let Some(fut) = future.as_mut().as_pin_mut() {
                ready!(fut.try_poll(cx))?;
                future.set(None);
            } else {
                match ready!(stream.as_mut().try_poll_next(cx)?) {
                    Some(e) => future.set(Some(f(e))),
                    None => break,
                }
            }
        }
        Poll::Ready(Ok(()))
    }
}
