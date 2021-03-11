use crate::event_bus::{EventBus, Request};
use conquer_once::Lazy;
use futures::lock::Mutex;
use message_types::bs_ps::{ToBackground, ToPopup};
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
    let msg = JsValue::from_serde(&ToBackground::BalanceRequest).unwrap();
    let response = JsFuture::from(browser.runtime().send_message(None, &msg, None)).await;
    log::trace!("Balance update received: {:?}", response);

    if let Ok(response) = response {
        if let Ok(ToPopup::BalanceResponse(balances)) = response.into_serde() {
            event_bus.send(Request::WalletBalanceUpdate(balances));
        }
    }
}

async fn update_wallet_status(mut event_bus: Dispatcher<EventBus>) {
    let msg = JsValue::from_serde(&ToBackground::BackgroundStatusRequest).unwrap();
    let response = JsFuture::from(browser.runtime().send_message(None, &msg, None)).await;
    log::trace!("Wallet status update received: {:?}", response);

    if let Ok(response) = response {
        if let Ok(background_status) = response.into_serde() {
            event_bus.send(Request::BackgroundStatus(background_status));
        }
    }
}
