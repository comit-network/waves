use crate::typed_js_future::TypedJsFuture;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// A wrapper type around the cache storage.
pub struct CacheStorage {
    inner: web_sys::CacheStorage,
}

/// A wrapper type around a specific cache.
///
/// This wrapper type allows us to provide easy-to-use async functions contrary to having to deal with promises all over the place.
pub struct Cache {
    inner: web_sys::Cache,
}

impl CacheStorage {
    pub async fn open(&self, name: &str) -> Result<Cache, JsValue> {
        let cache = TypedJsFuture::from(self.inner.open(name)).await?;

        Ok(Cache { inner: cache })
    }
}

impl Cache {
    pub async fn add_with_str(&self, url: &str) -> Result<(), JsValue> {
        JsFuture::from(self.inner.add_with_str(url)).await?;

        Ok(())
    }

    pub async fn match_with_str(&self, url: &str) -> Result<Option<web_sys::Response>, JsValue> {
        let response = JsFuture::from(self.inner.match_with_str(url)).await?;

        if response.is_undefined() {
            Ok(None)
        } else {
            Ok(Some(cast!(response)?))
        }
    }
}

impl From<web_sys::CacheStorage> for CacheStorage {
    fn from(inner: web_sys::CacheStorage) -> Self {
        Self { inner }
    }
}
