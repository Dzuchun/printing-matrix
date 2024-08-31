#![allow(clippy::module_name_repetitions)]
#![no_std]
#[cfg(feature = "stderror")]
extern crate std;

extern crate alloc;

use url::{PathSegmentsMut, Url};

pub mod executor;

pub mod request;

pub mod primitives;

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

pub struct ResponseParts {
    pub status_code: u16,
    pub body: alloc::string::String,
}
