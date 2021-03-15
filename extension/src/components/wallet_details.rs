use qrcode::{render::svg, QrCode};
use wallet::BalanceEntry;
use ybc::TileCtx::{Ancestor, Child, Parent};
use yew::{prelude::*, Component, ComponentLink, Html, Properties};

pub struct WalletDetails {
    link: ComponentLink<Self>,
    props: Props,
    withdraw_address: String,
}

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct Props {
    pub address: String,
    pub balances: Vec<BalanceEntry>,
    pub on_withdraw_all: Callback<String>,
    pub loading: bool,
}

pub enum Msg {
    UpdateWithdrawAddress(String),
    WithdrawAll,
}

impl Component for WalletDetails {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        WalletDetails {
            props,
            withdraw_address: "".to_string(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateWithdrawAddress(a) => {
                self.withdraw_address = a;
                false
            }
            Msg::WithdrawAll => {
                self.props.loading = true;
                self.props
                    .on_withdraw_all
                    .emit(self.withdraw_address.clone());
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props.address != props.address || !are_equal(&self.props.balances, &props.balances)
        {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let Props {
            address, balances, ..
        } = &self.props;
        let address_svg_string = QrCode::with_error_correction_level(address, qrcode::EcLevel::H)
            .map(|code| {
                code.render::<svg::Color>()
                    .max_dimensions(128, 128)
                    .min_dimensions(128, 128)
                    .build()
            })
            .unwrap();
        let address_svg = string_to_svg(&address_svg_string);
        // we need to provide a callback function to the input field. This one does nothing.
        let update = Callback::default();
        html! {
            <>
                <ybc::Tile ctx=Ancestor vertical=true>
                    <ybc::Tile ctx=Parent vertical=true>
                        <ybc::Tile ctx=Child classes="box">
                            <ybc::Title classes="is-5">
                                { "Address"  }
                            </ybc::Title>
                        </ybc::Tile>
                        <ybc::Tile ctx=Child classes="box is-centering">
                            <ybc::Media classes="media-center">
                                <ybc::Image classes="is-128x128">
                                    {address_svg}
                                </ybc::Image>
                            </ybc::Media>
                            <ybc::TextArea
                                name="address"
                                readonly=true
                                update={update}
                                classes="is-rounded has-fixed-size data-cy-wallet-address-text-field"
                                value={address}>
                            </ybc::TextArea>
                        </ybc::Tile>
                        <ybc::Tile>
                            <ybc::Tile ctx=Child classes="box">
                                <ybc::Field addons=true>
                                    <ybc::Control classes="has-icons-right is-expended">
                                        <ybc::Input
                                            name="address"
                                            value=self.withdraw_address.clone()
                                            update=self.link.callback(|e| Msg::UpdateWithdrawAddress(e))
                                            r#type=ybc::InputType::Text
                                            placeholder="Withdraw everything to">
                                        </ybc::Input>
                                    </ybc::Control>
                                    <ybc::Control>
                                        <ybc::Button
                                            onclick=self.link.callback(|_| Msg::WithdrawAll)
                                            loading=self.props.loading
                                            classes="is-primary">
                                            <ybc::Icon classes="is-small is-right">
                                                <i class="fas fa-share"></i>
                                            </ybc::Icon>
                                        </ybc::Button>
                                    </ybc::Control>
                                </ybc::Field>
                            </ybc::Tile>

                        </ybc::Tile>
                    </ybc::Tile>

                    <ybc::Tile ctx=Parent vertical=true>
                        { balances.iter().map(render_balances).collect::<Html>() }
                    </ybc::Tile>
                </ybc::Tile>
            </>
        }
    }
}

fn render_balances(balance: &BalanceEntry) -> Html {
    let balance_id = format!("data-cy-{}-balance-text-field", balance.ticker.clone());
    let balance_classes = format!("label {}", balance_id);
    // todo use enums
    let ticker_icon = match balance.ticker.as_str() {
        "L-BTC" => html! {<img width="32px" src={"./bitcoin.svg"} />},
        "USDt" => html! {<img width="32px" src={"./tether.svg"} />},
        default => html! {<label class="label">{default}</label>},
    };
    let value = format!("{:.4}", balance.value);
    html! {
        <>
            <ybc::Tile ctx=Parent>
                <ybc::Tile ctx=Child classes="box">
                    <ybc::Media>
                        <ybc::MediaLeft>
                            <ybc::Image classes="is-24x24">
                                {ticker_icon}
                            </ybc::Image>
                        </ybc::MediaLeft>
                        <ybc::MediaContent>
                            <label class={balance_classes}>{value}</label>
                        </ybc::MediaContent>
                    </ybc::Media>
                </ybc::Tile>
            </ybc::Tile>
        </>
    }
}

fn string_to_svg(svg: &str) -> Html {
    web_sys::window()
        .and_then(|window| window.document())
        .map_or_else(
            || {
                html! { <p>{ "Failed to resolve `document`." }</p> }
            },
            |document| match document.create_element("div") {
                Ok(div) => {
                    div.set_inner_html(svg);
                    yew::virtual_dom::VNode::VRef(div.into())
                }
                Err(e) => html! { <p>{ format!("{:?}", &e) }</p> },
            },
        )
}

fn are_equal(a: &[BalanceEntry], b: &[BalanceEntry]) -> bool {
    // TODO this is very inefficient, we could change to hashmaps instead
    for a1 in a.iter() {
        if !b.contains(a1) {
            return false;
        }
    }

    // if every single element of a was in b, we compare length
    a.len() == b.len()
}
