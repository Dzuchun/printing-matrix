use crate::ResponseParts;

pub trait Request {
    /// Type to be returned in case of successful execution
    type Response;
    /// Error type representing unexpected response format and/or content
    type ResponseError;
    /// An Iterator yielding query parameters.
    type QueryParamsIter: IntoIterator<Item = (Self::QueryParameterName, Self::QueryParameterValue)>;
    type QueryParameterName: AsRef<str> + Sized;
    type QueryParameterValue: AsRef<str> + Sized;
    /// An iterator yielding url path segments
    type PathSegmentsIter: IntoIterator<Item = Self::PathSegment>;
    type PathSegment: AsRef<str> + Sized;

    /// Must set endpoint for this request
    fn endpoint(&self) -> Self::PathSegmentsIter;

    fn method(&self) -> edge_http::Method;

    fn query_params(&self) -> Self::QueryParamsIter;

    fn generate_reponse(&self, parts: ResponseParts)
        -> Result<Self::Response, Self::ResponseError>;
}
