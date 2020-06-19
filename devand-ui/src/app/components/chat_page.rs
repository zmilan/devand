use crate::app::components::ChatInput;
use crate::app::services::ChatService;
use devand_core::chat::ChatMessage;
use devand_core::PublicUserProfile;
use yew::services::interval::{IntervalService, IntervalTask};
use yew::{prelude::*, Properties};

pub struct ChatPage {
    props: Props,
    #[allow(dead_code)]
    service: ChatService,
    state: State,
    link: ComponentLink<Self>,
    #[allow(dead_code)]
    poll_service: IntervalService,
    #[allow(dead_code)]
    poll_task: IntervalTask,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub chat_with: String,
    pub me: PublicUserProfile,
}

pub enum Msg {
    OtherUserLoaded(Option<PublicUserProfile>),
    AddMessages(Vec<ChatMessage>),
    SendMessage(String),
    Poll,
}

#[derive(Default)]
struct State {
    messages: Vec<ChatMessage>,
    other_user: Option<PublicUserProfile>,
}

impl Component for ChatPage {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let new_messages_callback = link.callback(Msg::AddMessages);
        let other_user_loaded_callback = link.callback(Msg::OtherUserLoaded);

        let mut service = ChatService::new(new_messages_callback, other_user_loaded_callback);

        service.load_other_user(&props.chat_with);

        let state = State::default();

        let mut poll_service = IntervalService::new();

        let poll_task = poll_service.spawn(
            std::time::Duration::from_secs(1),
            link.callback(|_| Msg::Poll),
        );

        Self {
            props,
            service,
            state,
            link,
            poll_service,
            poll_task,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::OtherUserLoaded(user) => {
                if let Some(user) = &user {
                    let me = self.props.me.id;
                    let other = user.id;
                    let members = vec![other, me];
                    self.service.load_history(members);
                } else {
                    // TODO Display error
                }
                self.state.other_user = user;
                true
            }
            Msg::AddMessages(messages) => {
                log::debug!("{:?}", messages);
                for msg in messages {
                    self.state.messages.push(msg);
                }
                true
            }
            Msg::SendMessage(txt) => {
                log::debug!("{}", txt);
                self.service.send_message(txt);
                true
            }
            Msg::Poll => {
                let last_message = self.state.messages.last();
                self.service.poll(last_message);
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let messages = self.state.messages.iter().map(|msg| {
            let from_me = msg.author == self.props.me.id;
            let from_me_class = if from_me {
                "devand-from-me"
            } else {
                "devand-from-other"
            };
            html! {
                <div class=("devand-chat-message-bubble", from_me_class)>
                    <span class=("devand-chat-message-txt")>{ &msg.txt }</span>
                    <span class=("devand-timestamp")>{ format!("{:?}", msg.created_at) }</span>
                </div>
            }
        });
        html! {
            <>
                <h1>{ format!("Chat with {}", self.props.chat_with) }</h1>
                <p>{ format!("WIP - chat with {} will be here", self.props.chat_with) }</p>
                <div class="devand-chat-container">
                    <div class="devand-chat-messages">
                        { for messages }
                    </div>
                    <div class="devand-chat-footer">
                        <ChatInput on_return=self.link.callback(Msg::SendMessage) />
                    </div>
                </div>
            </>
        }
    }
}
