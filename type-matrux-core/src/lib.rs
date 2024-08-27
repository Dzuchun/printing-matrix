use http::StatusCode;
use url::{PathSegmentsMut, Url};

mod executor;

mod request;

pub struct CannotBeABase(pub Url);

#[derive(Debug, Clone, derive_more::AsRef)]
pub struct BaseUrl(Url);

impl BaseUrl {
    fn try_new(url: Url) -> Result<Self, CannotBeABase> {
        if url.cannot_be_a_base() {
            return Err(CannotBeABase(url));
        }
        Ok(Self(url))
    }

    fn into_inner(self) -> Url {
        self.0
    }

    fn add_params<K: AsRef<str>, V: AsRef<str>>(&mut self, iter: impl IntoIterator<Item = (K, V)>) {
        let mut pairs = self.0.query_pairs_mut();
        for (k, v) in iter {
            pairs.append_pair(k.as_ref(), v.as_ref());
        }
    }

    fn path_segments_mut(&mut self) -> PathSegmentsMut<'_> {
        self.0
            .path_segments_mut()
            .expect("This url was validated to be a base")
    }
}

struct ResponseParts {
    status_code: StatusCode,
    body: String,
}
