use alloc::string::{String, ToString};
use alloc::vec::Vec;
use edge_http::Method;

use serde::Deserialize;
use type_matrux_core::primitives::{Avatar, DisplayName, Username};
use type_matrux_core::{
    primitives::{id::UserId, PageIndex, Relationships, FIRST_PAGE},
    request::Request,
};
use type_matrux_util::sow::ASow;

/// A query to `/api/users/info?name={name}&page={page}&withRelationships={relationships}`
#[derive(Debug)]
pub struct SearchUsers<const RELATIONSHIPS: bool = false> {
    name: ASow<'static, str>,
    page: PageIndex,
}

impl SearchUsers {
    /// Constructs a default-ish request, with provided name query, asking for first page and no
    /// relationships
    fn new(name: impl Into<ASow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            page: FIRST_PAGE,
        }
    }

    /// Defines a page request should ask for
    fn with_page(self, page: impl Into<PageIndex>) -> Self {
        Self {
            page: page.into(),
            ..self
        }
    }
}

#[sealed::sealed]
pub trait SearchUsersSpec {
    const RELATIONSHIPS: bool;
    type User;
}

#[sealed::sealed]
impl SearchUsersSpec for SearchUsers<false> {
    const RELATIONSHIPS: bool = false;
    type User = UserNoRel;
}

#[sealed::sealed]
impl SearchUsersSpec for SearchUsers<true> {
    const RELATIONSHIPS: bool = true;
    type User = UserRel;
}

impl<const RELATIONSHIPS: bool> Request for SearchUsers<RELATIONSHIPS>
where
    Self: SearchUsersSpec,
    for<'de> <Self as SearchUsersSpec>::User: Deserialize<'de>,
{
    type Response = Vec<<Self as SearchUsersSpec>::User>;

    type ResponseError = serde_json::Error;

    type QueryParameterName = &'static str;

    type QueryParameterValue = ASow<'static, str>;

    type PathSegment = &'static str;

    fn endpoint(&self) -> impl IntoIterator<Item = Self::PathSegment> {
        ["api", "users", "info"]
    }

    fn method(&self) -> edge_http::Method {
        Method::Get
    }

    fn query_params(
        &self,
    ) -> impl IntoIterator<Item = (Self::QueryParameterName, Self::QueryParameterValue)> {
        [
            ("name", self.name.clone()),
            ("page", self.page.0.to_string().into()),
            (
                "withRelationships",
                <Self as SearchUsersSpec>::RELATIONSHIPS.to_string().into(),
            ),
        ]
    }

    fn generate_reponse(
        &self,
        parts: type_matrux_core::ResponseParts,
    ) -> Result<Self::Response, Self::ResponseError> {
        serde_json::from_str(parts.body.as_str())
    }
}

/// Some information about user.
///
/// To get `relationships` field as well, use [SearchUsers<true>]
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct UserNoRel {
    #[serde(rename = "_id")]
    id: UserId,
    username: Username,
    display_name: DisplayName,
    avatar: Avatar,
}

/// Some information about user.
///
/// To opt out of `relationships` field, use [SearchUsers<false>]
#[derive(Debug, serde::Deserialize)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct UserRel {
    #[serde(rename = "_id")]
    id: UserId,
    username: Username,
    display_name: DisplayName,
    avatar: Avatar,
    relationships: Relationships,
}
