use std::{fmt::Display, str::FromStr};
use wasm_bindgen::JsValue;
use web_sys::window;

/// A wrapper type around the cache storage.
pub struct Storage {
    inner: web_sys::Storage,
}

impl Storage {
    pub fn local_storage() -> Result<Self, JsValue> {
        let storage = window()
            .ok_or_else(|| JsValue::from_str("failed to access window object"))?
            .local_storage()?
            .ok_or_else(|| JsValue::from_str("no local storage available"))?;

        Ok(storage.into())
    }

    pub fn get_item<T>(&self, name: &str) -> Result<Option<T>, JsValue>
    where
        T: FromStr,
        <T as FromStr>::Err: Display,
    {
        let value = self.inner.get_item(name)?;

        let value = match value {
            Some(value) => value,
            None => return Ok(None),
        };

        let t = T::from_str(&value).map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Some(t))
    }

    pub fn set_item<V>(&self, name: &str, value: V) -> Result<(), JsValue>
    where
        V: ToString,
    {
        self.inner.set_item(name, &value.to_string())?;

        Ok(())
    }
}

impl From<web_sys::Storage> for Storage {
    fn from(inner: web_sys::Storage) -> Self {
        Self { inner }
    }
}
