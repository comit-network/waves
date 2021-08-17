use crate::storage::Storage;
use anyhow::{Context, Result};

/// A wrapper type around the local storage acting as cache for http requests.
pub struct CacheStorage {
    inner: Storage,
}

// We assume that the javascript API's are threadsafe
unsafe impl Send for CacheStorage {}
unsafe impl Sync for CacheStorage {}

impl CacheStorage {
    pub fn new() -> Result<Self> {
        let local_storage = Storage::local_storage().with_context(|| "Could not open storage")?;
        Ok(Self {
            inner: local_storage,
        })
    }

    /// This function will fetch the provided URL and store the response body in local storage.
    /// It will fail if the response body is not a string.
    async fn add(&self, url: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let body = client.get(url).send().await?;
        let body_text = body
            .text()
            .await
            .with_context(|| "response is not a string")?;
        self.inner
            .set_item(url, &body_text)
            .with_context(|| format!("failed to add request for {} to storage", url))?;

        Ok(())
    }

    async fn match_with_str(&self, url: &str) -> Result<Option<Response>> {
        let maybe_response = self.inner.get_item(url)?;
        match maybe_response {
            None => Ok(None),
            Some(response) => Ok(Some(Response { inner: response })),
        }
    }

    /// Convenience function that first tries to look up the value in the storage and if it is not present adds and returns it.
    ///
    /// This function will always return a response IF the request was successful (2xx status code).
    /// Failed requests will never be added to the storage.
    pub async fn match_or_add(&self, url: &str) -> Result<Response> {
        Ok(match self.match_with_str(url).await? {
            Some(response) => response,
            None => {
                self.add(url).await?;
                self.match_with_str(url)
                    .await?
                    .context("no response in storage")?
            }
        })
    }
}

pub struct Response {
    inner: String,
}

impl Response {
    pub async fn text(&self) -> Result<String> {
        Ok(self.inner.clone())
    }
}
