use anyhow::{Context, Result};
use std::{error::Error as StdError, str::FromStr};
use web_sys::window;

use crate::LoanDetails;

/// A wrapper type around the cache storage.
pub struct Storage {
    inner: web_sys::Storage,
}

impl Storage {
    pub async fn get_open_loans(&self) -> Result<Vec<LoanDetails>> {
        let loans = match self
            .get_item::<String>("open_loans")
            .context("no key \"open_loans\" in local storage")?
        {
            Some(loans) => serde_json::from_str(&loans)?,
            None => Vec::<LoanDetails>::new(),
        };

        Ok(loans)
    }

    pub fn local_storage() -> Result<Self> {
        let storage = map_err_to_anyhow!(window()
            .context("failed to access window object")?
            .local_storage())?
        .context("no local storage available")?;

        Ok(storage.into())
    }

    pub fn get_item<T>(&self, name: &str) -> Result<Option<T>>
    where
        T: FromStr,
        <T as FromStr>::Err: StdError + Send + Sync + 'static,
    {
        let value = map_err_to_anyhow!(self.inner.get_item(name))
            .with_context(|| format!("Failed to get item from key {}", name))?;

        let value = match value {
            Some(value) => value,
            None => return Ok(None),
        };

        let t = T::from_str(&value).context("failed to parse item from string")?;

        Ok(Some(t))
    }

    pub fn set_item<V>(&self, name: &str, value: V) -> Result<()>
    where
        V: ToString,
    {
        map_err_to_anyhow!(self.inner.set_item(name, &value.to_string()))
            .with_context(|| format!("Failed to set item to key {}", name))?;

        Ok(())
    }

    pub fn remove_item(&self, name: &str) -> Result<()> {
        map_err_to_anyhow!(self.inner.remove_item(name))
            .with_context(|| format!("Failed to remove item from key {}", name))?;

        Ok(())
    }
}

impl From<web_sys::Storage> for Storage {
    fn from(inner: web_sys::Storage) -> Self {
        Self { inner }
    }
}
