use wallet::{Trade, TradeSide};
use wasm_bindgen_futures::spawn_local;
use ybc::TileCtx::{Ancestor, Child, Parent};
use yew::{prelude::*, Component, ComponentLink, Html, Properties};

pub struct TradeInfo {
    link: ComponentLink<Self>,
    props: Props,
    signing: bool,
    rejecting: bool,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub trade: Trade,
    pub on_confirm: Callback<()>,
    pub on_reject: Callback<()>,
}

pub enum Msg {
    Sign,
    Reject,
}

impl Component for TradeInfo {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        TradeInfo {
            link,
            props,
            signing: false,
            rejecting: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Sign => {
                self.signing = true;
                let submit = self.props.on_confirm.clone();
                spawn_local(async move {
                    submit.emit(());
                });

                true
            }
            Msg::Reject => {
                self.rejecting = true;
                let submit = self.props.on_reject.clone();
                spawn_local(async move {
                    submit.emit(());
                });

                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let Props {
            trade: Trade { sell, buy },
            ..
        } = &self.props;

        html! {
            <>
                <ybc::Subtitle>
                    { "Sign transaction"  }
                </ybc::Subtitle>
                <ybc::Tile ctx=Ancestor>
                    <ybc::Tile ctx=Parent vertical=true>
                        <ybc::Tile ctx=Parent vertical=true>
                            <p>{render_trade_side(sell, "You give")}</p>
                            <p>{render_trade_side(buy, "You receive")}</p>
                        </ybc::Tile>
                    </ybc::Tile>
                </ybc::Tile>
                <ybc::Button
                        onclick=self.link.callback(|_| Msg::Sign)
                        loading=self.signing
                        classes="is-primary data-cy-sign-tx-button">
                    { "Sign" }
                </ybc::Button>
                <ybc::Button
                        onclick=self.link.callback(|_| Msg::Reject)
                        loading=self.rejecting
                        classes="is-danger is-light data-cy-sign-tx-button">
                    { "Reject" }
                </ybc::Button>
            </>
        }
    }
}

fn render_trade_side(side: &TradeSide, action: &str) -> Html {
    let amount_str = format!("{:.2}", side.amount);
    let balance_before = format!("{:.2}", side.balance_before);
    let balance_after = format!("{:.2}", side.balance_after);
    html! {
        <>
            <ybc::Tile ctx=Child classes="box">
                <ybc::Subtitle>{ action  }</ybc::Subtitle>
            </ybc::Tile>
            <ybc::Tile ctx=Child classes="box">
                <p>{format!("{}: {}", side.ticker, amount_str)}</p>
                <p>{format!("Before: {}", balance_before)}</p>
                <p>{format!("After: {}", balance_after)}</p>
            </ybc::Tile>
        </>
    }
}
