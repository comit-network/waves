use message_types::bs_ps;
use yew::{prelude::*, Component, ComponentLink, Html, Properties};

pub struct WalletDetails {
    props: Props,
}

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct Props {
    pub address: String,
    pub balances: Vec<bs_ps::BalanceEntry>,
}

pub enum Msg {}

impl Component for WalletDetails {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        WalletDetails { props }
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props.address != props.address || !are_equal(&self.props.balances, &props.balances)
        {
            self.props = props;
            return true;
        } else {
            return false;
        }
    }

    fn view(&self) -> Html {
        let Props { address, balances } = &self.props;

        html! {
            <>
                <p>{"Address"}</p>
                <p data-cy="wallet-address-text-field">{address}</p>
                <p>{"Balances:"}</p>
                <ul class="item-list">
                    { balances.iter().map(render_balances).collect::<Html>() }
                </ul>
            </>
        }
    }
}

fn render_balances(balance: &bs_ps::BalanceEntry) -> Html {
    let balance_id = format!("{}-balance-text-field", balance.ticker.clone());
    html! {
        <li>
            <p>{balance.ticker.clone()} </p>
            <p data-cy={balance_id}>{balance.value.clone()}</p>
            </li>
    }
}

fn are_equal(a: &[bs_ps::BalanceEntry], b: &[bs_ps::BalanceEntry]) -> bool {
    // TODO this is very inefficient, we could change to hashmaps instead
    for a1 in a.iter() {
        if !b.contains(a1) {
            return false;
        }
    }

    // if every single element of a was in b, we compare length
    return a.len() == b.len();
}
