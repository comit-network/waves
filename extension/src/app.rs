use crate::{
    components::{CreateWallet, LoanInfo, TradeInfo, UnlockWallet, WalletDetails},
    event_bus::{EventBus, Response},
    wallet_updater::WalletUpdater,
};
use elements::Txid;
use js_sys::Promise;
use message_types::bs_ps::{
    BackgroundStatus, LoanData, SignState, ToBackground, TransactionData, WalletStatus,
};
use serde::{Deserialize, Serialize};
use wallet::{BalanceEntry, LoanDetails};
use wasm_bindgen::prelude::*;
use wasm_bindgen_extension::browser;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use yew::prelude::*;

// We do not support renaming the wallet for now
pub const WALLET_NAME: &str = "demo-wallet";

pub struct App {
    link: ComponentLink<Self>,
    state: State,
    _event_bus: Box<dyn Bridge<EventBus>>,
    _wallet_updater: WalletUpdater,
}

pub enum Msg {
    CreateWallet,
    UnlockWallet,
    BackgroundStatus(Box<BackgroundStatus>),
    BalanceUpdate(Vec<BalanceEntry>),
    SignAndSend { tx_hex: String, tab_id: u32 },
    Reject { tx_hex: String, tab_id: u32 },
    SignLoan { details: LoanDetails, tab_id: u32 },
    RejectLoan { details: LoanDetails, tab_id: u32 },
    WithdrawAll(String),
    RepayLoan(Txid),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    wallet_name: String,
    wallet_password: String,
    wallet_status: WalletStatus,
    wallet_balances: Vec<BalanceEntry>,
    sign_state: SignState,
    is_withdrawing: bool,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        log::debug!("PopupApp: creating...");

        let inner_link = link.clone();
        send_to_backend(
            ToBackground::BackgroundStatusRequest,
            Box::new(move |response| {
                if let Ok(response) = response {
                    if let Ok(msg) = response.into_serde() {
                        inner_link.send_message(Msg::BackgroundStatus(msg));
                    }
                }
            }),
        );

        let mut wallet_updater = WalletUpdater::new();
        wallet_updater.spawn();
        let callback = link.callback(|response| match response {
            Response::WalletBalanceUpdate(balances) => Msg::BalanceUpdate(balances),
            Response::BackgroundStatus(background_status) => {
                Msg::BackgroundStatus(Box::new(background_status))
            }
        });
        App {
            link,
            state: State {
                wallet_name: WALLET_NAME.to_string(),
                wallet_password: "".to_string(),
                wallet_status: WalletStatus::None,
                sign_state: SignState::None,
                wallet_balances: vec![],
                is_withdrawing: false,
            },
            _event_bus: EventBus::bridge(callback),
            _wallet_updater: wallet_updater,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UnlockWallet => {
                let inner_link = self.link.clone();
                send_to_backend(
                    ToBackground::UnlockRequest(
                        self.state.wallet_name.clone(),
                        self.state.wallet_password.clone(),
                    ),
                    Box::new(move |response| {
                        if let Ok(response) = response {
                            if let Ok(status) = response.into_serde() {
                                inner_link.send_message(Msg::BackgroundStatus(status));
                            }
                        }
                    }),
                );
                false
            }
            Msg::CreateWallet => {
                let inner_link = self.link.clone();
                let background_state =
                    BackgroundStatus::new(WalletStatus::NotLoaded, self.state.sign_state.clone());
                send_to_backend(
                    ToBackground::CreateWalletRequest(
                        self.state.wallet_name.clone(),
                        self.state.wallet_password.clone(),
                    ),
                    Box::new({
                        move |response| {
                            if response.is_ok() {
                                inner_link.send_message(Msg::BackgroundStatus(Box::new(
                                    background_state.clone(),
                                )));
                            }
                        }
                    }),
                );
                false
            }
            Msg::BackgroundStatus(status) => {
                self.state.wallet_status = status.wallet;
                self.state.sign_state = status.sign_state;

                true
            }
            Msg::SignAndSend { tx_hex, tab_id } => {
                let inner_link = self.link.clone();
                send_to_backend(
                    ToBackground::SignRequest { tx_hex, tab_id },
                    Box::new(move |response| {
                        if let Ok(response) = response {
                            if let Ok(status) = response.into_serde() {
                                inner_link.send_message(Msg::BackgroundStatus(status));
                            }
                        }
                    }),
                );
                false
            }
            Msg::Reject { tx_hex, tab_id } => {
                let inner_link = self.link.clone();
                send_to_backend(
                    ToBackground::Reject { tx_hex, tab_id },
                    Box::new(move |response| {
                        if let Ok(response) = response {
                            if let Ok(status) = response.into_serde() {
                                inner_link.send_message(Msg::BackgroundStatus(status));
                            }
                        }
                    }),
                );
                false
            }
            Msg::SignLoan { details, tab_id } => {
                let inner_link = self.link.clone();
                send_to_backend(
                    ToBackground::SignLoan { details, tab_id },
                    Box::new(move |response| {
                        if let Ok(response) = response {
                            if let Ok(status) = response.into_serde() {
                                inner_link.send_message(Msg::BackgroundStatus(status));
                            }
                        }
                    }),
                );
                false
            }
            Msg::RejectLoan { details, tab_id } => {
                let inner_link = self.link.clone();
                send_to_backend(
                    ToBackground::RejectLoan { details, tab_id },
                    Box::new(move |response| {
                        if let Ok(response) = response {
                            if let Ok(status) = response.into_serde() {
                                inner_link.send_message(Msg::BackgroundStatus(status));
                            }
                        }
                    }),
                );
                false
            }
            Msg::BalanceUpdate(balances) => {
                self.state.wallet_balances = balances;
                true
            }
            Msg::WithdrawAll(address) => {
                send_to_backend(
                    ToBackground::WithdrawAll(address),
                    Box::new(move |txid| {
                        if let Ok(txid) = txid {
                            if let Ok(txid) = txid.into_serde::<Txid>() {
                                log::debug!("Withdrawn everything to: {}", txid)
                            }
                        }
                    }),
                );
                self.state.is_withdrawing = false;
                true
            }
            Msg::RepayLoan(txid) => {
                let inner_link = self.link.clone();
                send_to_backend(
                    ToBackground::RepayLoan(txid),
                    Box::new(move |response| {
                        if let Ok(response) = response {
                            if let Ok(status) = response.into_serde() {
                                inner_link.send_message(Msg::BackgroundStatus(status));
                            }
                        }
                    }),
                );
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        true
    }

    fn view(&self) -> Html {
        let wallet_form = match self.state.clone() {
            State {
                wallet_status: WalletStatus::NotLoaded,
                ..
            } => {
                html! {
                    <UnlockWallet on_form_submit=self.link.callback(|_| Msg::UnlockWallet)></UnlockWallet>
                }
            }
            State {
                wallet_status: WalletStatus::None,
                ..
            } => {
                html! {
                    <CreateWallet on_form_submit=self.link.callback(|_| Msg::CreateWallet)></CreateWallet>
                }
            }
            State {
                wallet_status: WalletStatus::Loaded { .. },
                sign_state:
                    SignState::Trade(TransactionData {
                        hex,
                        decoded,
                        tab_id,
                    }),
                ..
            } => {
                let tx_hex = hex.clone();
                let sign_and_send = move |_| Msg::SignAndSend {
                    tx_hex: tx_hex.clone(),
                    tab_id,
                };
                let reject = move |_| Msg::Reject {
                    tx_hex: hex.clone(),
                    tab_id,
                };
                html! {

                    <>
                        <TradeInfo
                            trade=decoded
                            on_confirm=self.link.callback(sign_and_send)
                            on_reject=self.link.callback(reject)
                        >
                        </TradeInfo>
                    </>
                }
            }
            State {
                wallet_status: WalletStatus::Loaded { .. },
                sign_state: SignState::Loan(LoanData { details, tab_id }),
                ..
            } => {
                let sign_and_send = {
                    let details = details.clone();
                    move |_| Msg::SignLoan {
                        details: details.clone(),
                        tab_id,
                    }
                };

                let reject = {
                    let details = details.clone();
                    move |_| Msg::RejectLoan {
                        details: details.clone(),
                        tab_id,
                    }
                };
                html! {
                    <>
                        <LoanInfo
                            loan=details
                            on_confirm=self.link.callback(sign_and_send)
                            on_reject=self.link.callback(reject)
                        >
                        </LoanInfo>
                    </>
                }
            }
            State {
                wallet_status: WalletStatus::Loaded { address },
                sign_state: SignState::None,
                wallet_balances,
                ..
            } => {
                let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
                let open_loans = local_storage
                    .get_item("open_loans")
                    .unwrap()
                    .unwrap_or_default();
                let open_loans: Vec<Txid> = serde_json::from_str(&open_loans).unwrap_or_default();

                let loan_details = open_loans
                    .iter()
                    .map(|txid| {
                        let details = local_storage
                            .get_item(&format!("loan_details:{}", txid.to_string()))
                            .unwrap()
                            .unwrap();
                        let details = serde_json::from_str(&details).unwrap();

                        (details, *txid)
                    })
                    .collect::<Vec<(LoanDetails, Txid)>>();

                html! {
                    <><WalletDetails
                        address=address
                        balances=wallet_balances
                        loading=self.state.is_withdrawing
                        on_withdraw_all=self.link.callback(|address| Msg::WithdrawAll(address))
                     ></WalletDetails>
                        <ybc::Tile ctx=ybc::TileCtx::Parent vertical=true>
                    { loan_details.into_iter().map(|loan| {
                        render_open_loan(loan, self.link.clone())
                    }).collect::<Html>() }
                    </ybc::Tile>
                        </>
                }
            }
        };

        html! {
            <ybc::Section>
                <ybc::Container>
                    <ybc::Box>
                        { wallet_form }
                    </ybc::Box>
                </ybc::Container>
            </ybc::Section>
        }
    }

    fn rendered(&mut self, _first_render: bool) {}

    fn destroy(&mut self) {}
}

fn render_open_loan(loan: (LoanDetails, Txid), link: ComponentLink<App>) -> Html {
    html! {
        <ybc::Button
         onclick=link.callback(move |_| Msg::RepayLoan(loan.1))
         classes="is-primary repay-loan-button">{ "Repay loan " }
        </ybc::Button>
    }
}

fn send_to_backend(msg: ToBackground, callback: Box<dyn Fn(Result<JsValue, JsValue>)>) {
    spawn_local(async move {
        let js_value = JsValue::from_serde(&msg).unwrap();
        let promise: Promise = browser.runtime().send_message(None, &js_value, None);
        let result = JsFuture::from(promise).await;
        callback(result)
    });
}
