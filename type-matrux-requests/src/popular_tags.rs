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

    type QueryParamsIter = core::iter::Empty<(Self::QueryParameterName, Self::QueryParameterValue)>;

    type QueryParameterName = &'static str;

    type QueryParameterValue = &'static str;

    type PathSegmentsIter = core::array::IntoIter<Self::PathSegment, 4>;

    type PathSegment = &'static str;

    fn endpoint(&self) -> Self::PathSegmentsIter {
        ["api", "articles", "tags", "popular"].into_iter()
    }

    fn method(&self) -> Method {
        Method::Get
    }

    fn query_params(&self) -> Self::QueryParamsIter {
        std::iter::empty()
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
