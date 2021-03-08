use message_types::bs_ps::BackgroundStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use yew::worker::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    BackgroundStatusUpdate(Box<BackgroundStatus>),
}

pub struct EventBus {
    link: AgentLink<EventBus>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for EventBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Request;
    type Output = Box<BackgroundStatus>;

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
            Request::BackgroundStatusUpdate(update) => {
                for sub in self.subscribers.iter() {
                    let x = Box::new(BackgroundStatus {
                        wallet: update.wallet.clone(),
                        sign_tx: update.sign_tx.clone(),
                    });
                    self.link.respond(*sub, x);
                }
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
