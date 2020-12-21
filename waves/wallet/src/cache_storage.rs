use crate::typed_js_future::TypedJsFuture;
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
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
    pub fn from_window() -> Result<Self> {
        let window = web_sys::window().context("failed to get window handle")?;
        let cache_storage = map_err_to_anyhow!(window.caches())?;

        Ok(Self {
            inner: cache_storage,
        })
    }

    pub async fn open(&self, name: &str) -> Result<Cache> {
        let cache = map_err_to_anyhow!(TypedJsFuture::from(self.inner.open(name)).await)
            .with_context(|| format!("failed to open cache {}", name))?;

        Ok(Cache { inner: cache })
    }
}

impl Cache {
    pub async fn add_with_str(&self, url: &str) -> Result<()> {
        map_err_to_anyhow!(JsFuture::from(self.inner.add_with_str(url)).await)
            .with_context(|| format!("failed to add request for {} to cache", url))?;

        Ok(())
    }

    pub async fn match_with_str(&self, url: &str) -> Result<Option<Response>> {
        let response = map_err_to_anyhow!(JsFuture::from(self.inner.match_with_str(url)).await)
            .with_context(|| format!("failed to match request with url {}", url))?;

        if response.is_undefined() {
            Ok(None)
        } else {
            let response = map_err_to_anyhow!(cast!(response))?;

            Ok(Some(Response { inner: response }))
        }
    }

    /// Convenience function that first tries to look up the value in the cache and if it is not present adds and returns it.
    ///
    /// This function will always return a response IF the request was successful (2xx status code).
    /// Failed requests will never be added to the cache.
    pub async fn match_or_add(&self, url: &str) -> Result<Response> {
        Ok(match self.match_with_str(url).await? {
            Some(response) => response,
            None => {
                self.add_with_str(url).await?;
                self.match_with_str(url)
                    .await?
                    .context("no response in cache")?
            }
        })
    }
}

impl From<web_sys::CacheStorage> for CacheStorage {
    fn from(inner: web_sys::CacheStorage) -> Self {
        Self { inner }
    }
}

pub struct Response {
    inner: web_sys::Response,
}

impl Response {
    pub async fn json<T: DeserializeOwned>(&self) -> Result<T> {
        let promise = map_err_to_anyhow!(self.inner.json())?;
        let future = JsFuture::from(promise);

        let response = map_err_to_anyhow!(future.await)?;

        Ok(response
            .into_serde()
            .context("failed to deserialize response to json")?)
    }

    pub async fn text(&self) -> Result<String> {
        let promise = map_err_to_anyhow!(self.inner.text())?;
        let future = JsFuture::from(promise);

        Ok(map_err_to_anyhow!(future.await)?
            .as_string()
            .context("response is not a string")?)
    }
}
