use edge_http::Method;
use type_matrux_core::{
    primitives::{id::TagId, slug::TagSlug, TagMentions, TagName},
    request::Request,
    ResponseParts,
};

pub struct PopularTags;

impl Request for PopularTags {
    type Response = alloc::vec::Vec<Tag>;

    type ResponseError = serde_json::Error;

    type QueryParameterName = &'static str;

    type QueryParameterValue = &'static str;

    type PathSegment = &'static str;

    fn endpoint(&self) -> impl IntoIterator<Item = Self::PathSegment> {
        ["api", "articles", "tags", "popular"]
    }

    fn method(&self) -> Method {
        Method::Get
    }

    fn query_params(
        &self,
    ) -> impl IntoIterator<Item = (Self::QueryParameterName, Self::QueryParameterValue)> {
        core::iter::empty()
    }

    fn generate_reponse(
        &self,
        parts: ResponseParts,
    ) -> Result<Self::Response, Self::ResponseError> {
        serde_json::from_str(parts.body.as_str())
    }
}

#[derive(Debug, serde::Deserialize)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct Tag {
    #[serde(rename = "_id")]
    id: TagId,
    name: TagName,
    slug: TagSlug,
    #[serde(rename = "mentionsNum")]
    mentions_num: TagMentions,
}
