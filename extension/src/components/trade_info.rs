use wallet::{Trade, TradeSide};
use yew::{prelude::*, Component, ComponentLink, Html, Properties};

pub struct TradeInfo {
    props: Props,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub trade: Trade,
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

    fn change(&mut self, _props: Self::Properties) -> bool {
        true
    }

    fn view(&self) -> Html {
        let Props {
            trade: Trade { sell, buy },
        } = &self.props;

        html! {
            <>
                <p>{render_trade_side(sell, "Selling")}</p>
                <p>{render_trade_side(buy, "Buying")}</p>
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
