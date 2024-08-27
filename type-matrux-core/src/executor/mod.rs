use core::fmt::Display;
use http::Method;
use url::Url;

use crate::{request::Request, BaseUrl, ResponseParts};

#[derive(Debug)]
pub enum ExecutorError<E, R> {
    Execution(E),
    Response(R),
}

impl<E: Display, R: Display> Display for ExecutorError<E, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutorError::Execution(err) => err.fmt(f),
            ExecutorError::Response(err) => err.fmt(f),
        }
    }
}

/// A low-level request executor. Requires defining a single method: `send`. This is a heart of this
/// API: all of the requests you send are relayed to it's implementation as some point
///
/// Executor must be able to create itself from a base site url
pub trait RequestExecutor: RequestExecutorInner + Sized {
    /// An error that can occur while `Client` setups itself.
    ///
    /// Client may or may not check url validity, site availability, or whatever.
    type CreationError: Sized;

    fn create(base: BaseUrl) -> Result<Self, Self::CreationError>;

    async fn send<R: Request>(
        &self,
        request: R,
    ) -> Result<R::Response, ExecutorError<Self::ExecutionError, R::ResponseError>> {
        // Create base url
        let mut url = self.base_url().clone();
        // Append some path to it
        url.path_segments_mut().extend(request.endpoint());
        // Add query parameters
        url.add_params(request.query_params());
        // Execute the request
        let parts = self
            .send_inner(url.into_inner(), request.method())
            .await
            .map_err(ExecutorError::Execution)?;
        // Construct the response
        let response = request
            .generate_reponse(parts)
            .map_err(ExecutorError::Response)?;
        Ok(response)
    }
}

pub(crate) trait RequestExecutorInner {
    type ExecutionError: Sized;

    fn base_url(&self) -> &BaseUrl;

    async fn send_inner(
        &self,
        url: Url,
        method: Method,
    ) -> Result<ResponseParts, Self::ExecutionError>;
}
