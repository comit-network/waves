use crate::event_bus::{EventBus, Request};
use conquer_once::Lazy;
use futures::lock::Mutex;
use js_sys::Promise;
use message_types::{bs_ps, Component as MessageComponent};
use wasm_bindgen::JsValue;
use wasm_bindgen_extension::browser;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use yew::{agent::Dispatcher, Dispatched};

static RUN: Lazy<Mutex<Option<bool>>> = Lazy::new(|| Mutex::new(Some(true)));
// TODO: in production setting 5 seconds polling is a bit much
const UPDATE_INTERVAL_SEC: u64 = 5;

#[derive(Clone)]
pub struct WalletUpdater {}

impl WalletUpdater {
    pub fn new() -> Self {
        WalletUpdater {}
    }

    pub fn spawn(&mut self) {
        let future = async {
            {
                while RUN.lock().await.unwrap() {
                    update_wallet_balance(EventBus::dispatcher()).await;
                    update_wallet_status(EventBus::dispatcher()).await;
                    futures_timer::Delay::new(std::time::Duration::from_secs(UPDATE_INTERVAL_SEC))
                        .await;
                }
            }
        };
        spawn_local(future);
    }
}

impl Drop for WalletUpdater {
    fn drop(&mut self) {
        let stop = async {
            RUN.lock().await.replace(false);
        };
        spawn_local(stop)
    }
}

async fn update_wallet_balance(mut event_bus: Dispatcher<EventBus>) {
    let msg = bs_ps::Message {
        rpc_data: bs_ps::RpcData::GetBalance,
        target: MessageComponent::Background,
        source: MessageComponent::PopUp,
    };

    let js_value = JsValue::from_serde(&msg).unwrap();
    let promise: Promise = browser.runtime().send_message(None, &js_value, None);
    let response = JsFuture::from(promise).await;
    log::trace!("Wallet status update received: {:?}", response);

    if let Ok(response) = response {
        if let Ok(bs_ps::RpcData::Balance(balances)) = response.into_serde() {
            event_bus.send(Request::WalletBalanceUpdate(balances));
        }
    }
}

async fn update_wallet_status(mut event_bus: Dispatcher<EventBus>) {
    let msg = bs_ps::Message {
        rpc_data: bs_ps::RpcData::GetWalletStatus,
        target: MessageComponent::Background,
        source: MessageComponent::PopUp,
    };

    let js_value = JsValue::from_serde(&msg).unwrap();
    let promise: Promise = browser.runtime().send_message(None, &js_value, None);
    let response = JsFuture::from(promise).await;
    log::trace!("Wallet status update received: {:?}", response);

    if let Ok(response) = response {
        if let Ok(background_status) = response.into_serde() {
            event_bus.send(Request::BackgroundStatus(background_status));
        }
    }
}
