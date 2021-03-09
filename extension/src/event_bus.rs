use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use wallet::BalanceEntry;
use yew::worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    WalletBalanceUpdate(Vec<BalanceEntry>),
}

pub struct EventBus {
    link: AgentLink<EventBus>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for EventBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Request;
    type Output = Vec<BalanceEntry>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Request::WalletBalanceUpdate(balances) => {
                for sub in self.subscribers.iter() {
                    let balances = balances.clone();
                    self.link.respond(*sub, balances);
                }
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
