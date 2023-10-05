use std::{num::NonZeroUsize, pin::Pin, task::Poll};

use futures::{Future, Stream};
use pin_project::pin_project;

use crate::DrukarniaApi;

use super::Res;

type Fut<'l, E> = Pin<Box<dyn Future<Output = Res<Vec<E>>> + 'l>>;

#[pin_project]
pub struct PageSearchStream<'client, 'generator, 'future, Auth, E> {
    pub(super) client: &'client dyn DrukarniaApi<Auth = Auth>,
    pub(super) generator: Box<dyn (Fn(NonZeroUsize) -> Fut<'future, E>) + 'generator>,
    pub(super) current_page: NonZeroUsize,
    #[pin]
    current_future: Fut<'future, E>,
    errored: bool,
}

impl<'client, 'generator, 'future, Auth, E>
    PageSearchStream<'client, 'generator, 'future, Auth, E>
{
    pub(super) fn create<G>(client: &'client dyn DrukarniaApi<Auth = Auth>, generator: G) -> Self
    where
        'client: 'generator,
        'generator: 'future,
        G: (Fn(NonZeroUsize) -> Fut<'future, E>) + 'generator,
    {
        let first_page: NonZeroUsize = NonZeroUsize::new(1).expect("1 != 0");
        Self {
            current_future: generator(first_page),
            client,
            generator: Box::new(generator),
            current_page: first_page,
            errored: false,
        }
    }

    pub fn flat(self) -> SearchStream<'client, 'generator, 'future, Auth, E> {
        SearchStream {
            parent: self,
            this_page: vec![],
        }
    }
}

impl<'generator, 'future, 'client, Auth, E> Stream
    for PageSearchStream<'client, 'generator, 'future, Auth, E>
where
    'client: 'generator,
    'generator: 'future,
{
    type Item = Res<Vec<E>>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut projection = self.project();
        if *projection.errored {
            // API had errored previously, end the stream
            return Poll::Ready(None);
        }

        match projection.current_future.as_mut().poll(cx) {
            Poll::Ready(res) => {
                match res {
                    Ok(ok) => {
                        if ok.is_empty() {
                            // Results had ended, and so is this stream
                            Poll::Ready(None)
                        } else {
                            // Next page fetched successfully
                            // Step up the page
                            *projection.current_page = projection.current_page.saturating_add(1);
                            // Create new future
                            projection
                                .current_future
                                .set((projection.generator)(*projection.current_page));
                            // Return current result
                            Poll::Ready(Some(Ok(ok)))
                        }
                    }
                    Err(err) => {
                        // API had errored
                        // Return the error now, but flip the flag, so that on next poll stream would end
                        *projection.errored = true;
                        Poll::Ready(Some(Err(err)))
                    }
                }
            }
            Poll::Pending => {
                // Next page was not loaded yet
                Poll::Pending
            }
        }
    }
}

#[pin_project]
pub struct SearchStream<'client, 'generator, 'future, Auth, E> {
    #[pin]
    parent: PageSearchStream<'client, 'generator, 'future, Auth, E>,
    this_page: Vec<E>,
}

impl<'client, 'generator, 'future, Auth, E> Stream
    for SearchStream<'client, 'generator, 'future, Auth, E>
where
    'client: 'generator,
    'generator: 'future,
{
    type Item = Res<E>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let projection = self.project();
        if projection.this_page.is_empty() {
            let mut new_page = match projection.parent.poll_next(cx) {
                Poll::Ready(Some(Ok(page))) => page,
                Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(err))),
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };

            new_page.reverse();
            *projection.this_page = new_page;
        }
        // This page is not empty now
        if let Some(article) = projection.this_page.pop() {
            Poll::Ready(Some(Ok(article)))
        } else {
            Poll::Ready(None)
        }
    }
}
