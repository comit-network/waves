# Architecture

You are looking at a WebExtension implemented in TypeScript with some components written in Rust and compiled to WASM.

## Components

Being a WebExtension, we have several components that interact with each other:

- A background script:
  Executed as soon as the extension is loaded, remains alive until the extension is unloaded / uninstalled or the browser is closed.
  Background scripts have the most privileges and can open windows etc but cannot directly communicate with browser tabs.

- A content script:
  Shipped with the extension, content script are injected into tabs and are hence bound to their lifetime.
  Content scripts can communicate with background scripts via message passing and can inject scripts into web pages.

- An in-page script:
  Also shipped with the extension, in-page scripts are injected as normal scripts into a web-page and can access the JavaScript context of the web page.
  They can communicate with content-script via message passing.

- A popup window:
  The popup window that is shown on the click of the extension button is a web page whose JavaScript context has the same privileges as a background script.
  Compared to a background script however, its lifetime is limited to the visibility of the window.
  As soon as the window is dismissed, its JavaScript context is destroyed.

## Design

We implement the core functionality of the wallet and our protocols in Rust.
The resulting Rust library is compiled to WASM.
Importing this WASM blob in the background script allows the WASM code to run with the same privileges as the background script itself.
Using the `web_sys` library, the Rust code can interact with the browser's APIs that are made available to background scripts.

From the perspective of all other components, the use of Rust for implementing the wallet should be an implementation of the background script.

## APIs

We differentiate between two sets of APIs that are exposed by the background script:

1. Webpage facing APIs
2. Extension facing APIs

APIs exposed to the web pages need to be accessed via the in-page script and the content script.
The content script can only communicate with the background script via message passing.
As such, all APIs that should be accessible from within web pages are defined as message handlers.
These message handlers can either be implemented in Rust or TypeScript.

Privileged scripts like the popup script or other background scripts can access the state of the background script directly.
Hence, any functionality that is _internal_ to the extension and not exposed to the web pages is made available as functions on the `window` object of the background page.
These functions can again be defined in Rust and TypeScript.

## Invariants

### Promise within `browser.runtime.onMessage.addListener` must not be used for control flow

The message passing between content-script and background-script happens via the `browser.runtime.sendMessage` API.
It is important that these promises **MUST NOT** be used for control flow.
See https://github.com/mozilla/webextension-polyfill/issues/228#issuecomment-623982728.
In other words, even if the response to such a `sendMessage` call is an error, the promise returned from `browser.runtime.onMessage.addListener` should still be _resolved_ and **not** rejected.

### Background-script must never be imported

The background script initializes all sorts of stuff when it is loaded.
Most importantly, it imports the WASM blob which initializes the state for the wallet.
This happens as soon as the extension is loaded.

To ensure that all of this only ever happens once, we **MUST NOT** import anything from the background script (i.e. the `index.ts` file).
For this reason, we have a separate `api.ts` file within the `background/` directory that holds all the types and exports that other components can use to talk to the background script.

**tl;dr:** The background script (`background/index.ts`) **MUST NOT** export anything.
