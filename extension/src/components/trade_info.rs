use wallet::{Trade, TradeSide};
use yew::{prelude::*, Component, ComponentLink, Html, Properties};

pub struct TradeInfo {
    props: Props,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub trade: Trade,
    pub on_form_submit: Callback<()>,
}

pub enum Msg {}

impl Component for TradeInfo {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        TradeInfo { props }
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
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
            on_form_submit,
        } = &self.props;

        let onclick = on_form_submit.reform(move |_| ());

        html! {
            <>
                <p>{"Sign transaction"}</p>
                <p>{render_trade_side(sell, "Selling")}</p>
                <p>{render_trade_side(buy, "Buying")}</p>
                <button data-cy="sign-tx-button" onclick={onclick}>{ "Sign" }</button>
            </>
        }
    }
}

fn render_trade_side(side: &TradeSide, action: &str) -> Html {
    html! {
        <>

            <p>{format!("{} {}{}", action, side.amount, side.ticker)}</p>
            <p>{format!("{} balance: {} -> {}", side.ticker, side.balance_before, side.balance_after)}</p>
        </>
    }
}
