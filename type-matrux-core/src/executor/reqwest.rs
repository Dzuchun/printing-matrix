use std::{convert::Infallible, sync::Arc};

use http::{header, Method};
use reqwest::Client;
use url::Url;

use crate::{BaseUrl, ResponseParts};

use super::{RequestExecutor, RequestExecutorInner};

static USER_AGENT: &str = "type-matrux/0.2.0";
static DRUKARNIA_SITE: &str = "drukarnia.com.ua";

#[cfg_attr(feature = "stderror", derive(thiserror::Error))]
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum ReqwestExecutorError {
    #[cfg_attr(feature = "stderror", error("Unknown request method: {0}"))]
    UnknownMethod(Method),
    #[cfg_attr(feature = "stderror", error("Error while performing request: {0}"))]
    Send(reqwest::Error),
    #[cfg_attr(feature = "stderror", error("Error while decoding response: {0}"))]
    Decode(reqwest::Error),
}

#[allow(clippy::module_name_repetitions)]
pub struct ReqwestExecutor {
    client: Client,
    base: Arc<BaseUrl>,
}

impl RequestExecutor for ReqwestExecutor {
    type CreationError = Infallible;

    fn create(base: BaseUrl) -> Result<Self, Self::CreationError> {
        Ok(Self {
            client: Client::new(),
            base: Arc::new(base),
        })
    }
}

impl RequestExecutorInner for ReqwestExecutor {
    type ExecutionError = ReqwestExecutorError;

    fn base_url(&self) -> &BaseUrl {
        &self.base
    }

    async fn send_inner(
        &self,
        url: Url,
        method: Method,
    ) -> Result<ResponseParts, Self::ExecutionError> {
        // Start by selecting correct method
        let builder = if method == Method::GET {
            self.client.get(url)
        } else if method == Method::POST {
            self.client.post(url)
        } else {
            return Err(ReqwestExecutorError::UnknownMethod(method));
        };
        // Append necessary headers
        let builder = builder
            .header(header::USER_AGENT, USER_AGENT)
            .header(header::HOST, DRUKARNIA_SITE);
        // Execute the request
        let response = builder.send().await.map_err(ReqwestExecutorError::Send)?;
        let status_code = response.status();
        let body = response
            .text()
            .await
            .map_err(ReqwestExecutorError::Decode)?;
        Ok(ResponseParts { status_code, body })
    }
}
