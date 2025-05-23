use serde::{Deserialize, Serialize};
use web_sys::{HtmlInputElement, Storage, window};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    ToggleDarkMode,
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

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    dark_mode: bool,
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

        // Load dark mode preference from local storage
        let dark_mode = window()
            .and_then(|win| win.local_storage().ok())
            .flatten()
            .and_then(|storage| storage.get_item("dark_mode").ok())
            .flatten()
            .map(|value| value == "true")
            .unwrap_or(false);

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
            dark_mode,
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
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
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
            Msg::ToggleDarkMode => {
                self.dark_mode = !self.dark_mode;
                
                // Save preference to local storage
                if let Some(storage) = window().and_then(|win| win.local_storage().ok()).flatten() {
                    let _ = storage.set_item("dark_mode", &self.dark_mode.to_string());
                }
                
                true
            }
        }
    }
    
    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let toggle_dark_mode = ctx.link().callback(|_| Msg::ToggleDarkMode);
        
        // Define classes based on dark mode state
        let main_bg = if self.dark_mode { "bg-gray-900 text-white" } else { "bg-white text-black" };
        let sidebar_bg = if self.dark_mode { "bg-gray-800" } else { "bg-gray-100" };
        let user_card_bg = if self.dark_mode { "bg-gray-700" } else { "bg-white" };
        let border_color = if self.dark_mode { "border-gray-700" } else { "border-gray-300" };
        let message_bubble_bg = if self.dark_mode { "bg-gray-800" } else { "bg-gray-100" };
        let input_bg = if self.dark_mode { "bg-gray-700 text-white placeholder-gray-400" } else { "bg-gray-100 text-gray-700 placeholder-gray-500" };
        let header_text = if self.dark_mode { "text-blue-400" } else { "text-blue-600" };
        let username_text = if self.dark_mode { "text-blue-300" } else { "text-blue-600" };
        let message_text = if self.dark_mode { "text-gray-300" } else { "text-gray-600" };
        
        html! {
            <div class={format!("flex w-screen {}", main_bg)}>
                <div class={format!("flex-none w-56 h-screen {}", sidebar_bg)}>
                    <div class="flex justify-between items-center p-3">
                        <div class={format!("text-xl {}", header_text)}>{"Users"}</div>
                        <button 
                            onclick={toggle_dark_mode} 
                            class="p-2 rounded-lg hover:bg-opacity-80"
                            title={if self.dark_mode { "Switch to Light Mode" } else { "Switch to Dark Mode" }}
                        >
                            {
                                if self.dark_mode {
                                    // Sun icon for light mode
                                    html! {
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-yellow-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
                                        </svg>
                                    }
                                } else {
                                    // Moon icon for dark mode
                                    html! {
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-gray-700" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
                                        </svg>
                                    }
                                }
                            }
                        </button>
                    </div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class={format!("flex m-3 {} rounded-lg p-2", user_card_bg)}>
                                    <div>
                                        <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-xs justify-between">
                                            <div class={username_text.clone()}>{u.name.clone()}</div>
                                        </div>
                                        <div class="text-xs text-gray-400">
                                            {"Hi there!"}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col">
                    <div class={format!("w-full h-14 border-b-2 {}", border_color)}>
                        <div class={format!("text-xl p-3 {}", header_text)}>{"💬 Chat!"}</div>
                    </div>
                    <div class={format!("w-full grow overflow-auto border-b-2 {}", border_color)}>
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html!{
                                    <div class={format!("flex items-end w-3/6 {} m-8 rounded-tl-lg rounded-tr-lg rounded-br-lg", message_bubble_bg)}>
                                        <img class="w-8 h-8 rounded-full m-3" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="p-3">
                                            <div class={format!("text-sm {}", username_text)}>
                                                {m.from.clone()}
                                            </div>
                                            <div class={format!("text-xs {}", message_text)}>
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-3" src={m.message.clone()}/>
                                                } else {
                                                    {m.message.clone()}
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="w-full h-14 flex px-3 items-center">
                        <input 
                            ref={self.chat_input.clone()} 
                            type="text" 
                            placeholder="Message" 
                            class={format!("block w-full py-2 pl-4 mx-3 {} rounded-full outline-none", input_bg)} 
                            name="message" 
                            required=true 
                        />
                        <button 
                            onclick={submit} 
                            class="p-3 shadow-sm bg-blue-600 w-10 h-10 rounded-full flex justify-center items-center hover:bg-blue-700"
                        >
                            <svg fill="#000000" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white">
                                <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}