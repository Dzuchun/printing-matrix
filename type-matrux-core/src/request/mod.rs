use crate::ResponseParts;

pub trait Request {
    /// Type to be returned in case of successful execution
    type Response;
    /// Error type representing unexpected response format and/or content
    type ResponseError;
    type QueryParameterName: AsRef<str> + Sized;
    type QueryParameterValue: AsRef<str> + Sized;
    type PathSegment: AsRef<str> + Sized;

    /// Must set endpoint for this request
    fn endpoint(&self) -> impl IntoIterator<Item = Self::PathSegment>;

    fn method(&self) -> edge_http::Method;

    fn query_params(
        &self,
    ) -> impl IntoIterator<Item = (Self::QueryParameterName, Self::QueryParameterValue)>;

    fn generate_reponse(&self, parts: ResponseParts)
        -> Result<Self::Response, Self::ResponseError>;
}
