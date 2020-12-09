use elements_fun::bitcoin_hashes::core::pin::Pin;
use futures::{
    task::{Context, Poll},
    FutureExt,
};
use js_sys::Promise;
use std::{future::Future, marker::PhantomData};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// A wrapper around [`JsFuture`] that automatically casts the resolved value to a specific type.
pub struct TypedJsFuture<T> {
    inner: JsFuture,
    phantom: PhantomData<T>,
}

impl<T> TypedJsFuture<T> {
    pub fn from(promise: Promise) -> Self {
        Self {
            inner: JsFuture::from(promise),
            phantom: PhantomData,
        }
    }
}

impl<T> Future for TypedJsFuture<T>
where
    T: JsCast,
    Self: Unpin,
{
    type Output = Result<T, JsValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let js_value = futures::ready!(self.inner.poll_unpin(cx))?;

        Poll::Ready(cast!(js_value))
    }
}
