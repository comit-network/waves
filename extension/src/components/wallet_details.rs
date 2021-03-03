use message_types::bs_ps;
use yew::{prelude::*, Component, ComponentLink, Html, Properties};

pub struct WalletDetails {
    props: Props,
}

#[derive(Properties, Clone)]
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

    fn change(&mut self, _props: Self::Properties) -> bool {
        true
    }

    fn view(&self) -> Html {
        let Props { address, balances } = &self.props;

        html! {
            <>
                <p>{format!("Address: {}", address)}</p>
                <p>{"Balances:"}</p>
                <ul class="item-list">
                    { balances.iter().map(render_balances).collect::<Html>() }
                </ul>
            </>
        }
    }
}

fn render_balances(balance: &bs_ps::BalanceEntry) -> Html {
    html! {
        <li> {balance.ticker.clone()}  {balance.value.clone()}  </li>
    }
}
