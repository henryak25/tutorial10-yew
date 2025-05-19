use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use crate::services::event_bus::EventBus;
use crate::{User, services::websocket::WebsocketService};
pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
}


pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: "/joebiden.png".into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    //log::debug!("got input: {:?}", input.value());
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        html! {
<div class="flex w-screen min-h-screen bg-gray-900">
    <div class="flex-none w-56 h-screen bg-gray-800 shadow-lg">
        <div class="text-xl p-3 font-semibold border-b border-gray-700 text-gray-200">{"Users"}</div>
        {
            self.users.clone().iter().map(|u| {
                html!{
                    <div class="flex items-center gap-3 m-3 bg-gray-700 rounded-xl p-3 shadow-md hover:shadow-lg hover:bg-gray-600 hover:scale-[1.02] active:scale-95 active:bg-blue-900 transition-all duration-200 ease-in-out cursor-pointer transform group">
                        <img class="w-12 h-12 rounded-full border-2 border-blue-500 group-hover:border-blue-400 transition-colors" src={u.avatar.clone()} alt="avatar"/>
                        <div class="flex-grow">
                            <div class="text-sm font-semibold text-gray-100 group-hover:text-white">{u.name.clone()}</div>
                            <div class="text-xs text-gray-400 italic group-hover:text-gray-300">{"Hi there!"}</div>
                        </div>
                    </div>
                }
            }).collect::<Html>()
        }
    </div>
    <div class="grow h-screen flex flex-col bg-gray-900 shadow-inner">
        <div class="w-full h-14 border-b-2 border-gray-800 flex items-center px-4 bg-gray-800">
            <div class="text-xl font-semibold text-gray-200">{"💬 Chat!"}</div>
        </div>
        <div class="w-full grow overflow-auto border-b-2 border-gray-800 p-4 bg-gray-900">
            {
                self.messages.iter().map(|m| {
                    let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                    html!{
                        <div class="flex items-end max-w-3xl bg-gray-800 m-4 rounded-tl-lg rounded-tr-lg rounded-br-lg shadow-sm hover:bg-gray-700 transition-colors duration-150">
                            <img class="w-8 h-8 rounded-full m-3 border border-gray-600" src={user.avatar.clone()} alt="avatar"/>
                            <div class="p-3">
                                <div class="text-sm font-semibold text-blue-400">
                                    {m.from.clone()}
                                </div>
                                <div class="text-sm text-gray-300 mt-1">
                                    {
                                        if m.message.ends_with(".gif") {
                                            html! {
                                                <img class="mt-3 max-w-xs rounded-md border border-gray-600" src={m.message.clone()} alt="gif message"/>
                                            }
                                        } else {
                                            html! { m.message.clone() }
                                        }
                                    }
                                </div>
                            </div>
                        </div>
                    }
                }).collect::<Html>()
            }
        </div>
        <div class="w-full h-16 flex px-3 items-center bg-gray-800 border-t border-gray-700">
            <input
                ref={self.chat_input.clone()}
                type="text"
                placeholder="Message"
                class="block w-full py-2 pl-4 mx-3 bg-gray-700 rounded-full outline-none text-gray-200 placeholder-gray-400 focus:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:bg-gray-600 transition-all"
                name="message"
                required=true
            />
            <button
                onclick={submit}
                class="p-3 bg-blue-600 w-10 h-10 rounded-full flex justify-center items-center text-white hover:bg-blue-500 hover:scale-105 active:scale-95 transition-all shadow-lg hover:shadow-blue-500/30"
                aria-label="Send message"
            >
                <svg fill="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white">
                    <path d="M0 0h24v24H0z" fill="none"></path>
                    <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                </svg>
            </button>
        </div>
    </div>
</div>

        }
    }

}
