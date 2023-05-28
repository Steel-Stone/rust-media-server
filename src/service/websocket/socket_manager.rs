use super::messaging::{answer_generator::AnswerGenerator, pinger_job::PingerJob};
use crate::clients::public_ip;
use crate::server_constants;
use crate::service::websocket::signal_connection_maker::SignalConnectionMaker;
use async_std::task;
use serde::Deserialize;
use std::net::TcpStream;
use std::time::Duration;
use tungstenite::{stream::MaybeTlsStream, Error::Io, Message, WebSocket};

type ResultOrError<T, E> = std::result::Result<T, E>;

pub struct SocketManager<'a> {
    answer_generator: Option<AnswerGenerator>,
    pinger_job: Option<PingerJob<'a>>,
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    socket_maker: SignalConnectionMaker,
}

#[derive(Debug, Deserialize)]
struct SDPOffer {
    description: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum IncomingMessage {
    #[serde(rename = "offer")]
    SDPOffer(SDPOffer),
    #[serde(rename = "pong")]
    Pong,
}

#[derive(strum_macros::Display)]
pub enum OutgoingType {
    Answer,
    Error,
}

pub struct OutgoingMessage {
    pub message_type: OutgoingType,
    pub message: String,
}

impl<'a> SocketManager<'a> {
    pub fn new(socket_maker: SignalConnectionMaker) -> SocketManager<'a> {
        // TODO keep trying to connect if it results in an error
        let socket = socket_maker.connect_to_signaling().unwrap();
        SocketManager {
            answer_generator: None,
            pinger_job: None,
            socket,
            socket_maker,
        }
    }

    pub fn set_answer_generator(
        &mut self,
        answer_generator: AnswerGenerator,
    ) -> &mut SocketManager<'a> {
        self.answer_generator = Some(answer_generator);
        self
    }

    pub fn set_pinger_job(&mut self, pinger_job: PingerJob<'a>) -> &mut SocketManager<'a> {
        self.pinger_job = Some(pinger_job);
        self
    }

    pub async fn listen(&mut self) {
        //TODO : Have a wrapper that converts the websocket Message to our MessageType Enum
        loop {
            match SocketManager::blocking_listen(self).await {
                Ok(_) => panic!("SocketManager Listener returned unexpected OK"),
                Err(_) => {
                    // TODO bring this out into its own function
                    task::sleep(Duration::from_secs(5)).await;
                    let socket_result = self.socket_maker.connect_to_signaling();
                    if socket_result.is_err() {
                        continue;
                    }
                    self.socket = socket_result.unwrap();
                }
            }
        }
    }

    async fn blocking_listen(&mut self) -> ResultOrError<(), std::io::Error> {
        loop {
            let msg = self.socket.read_message();
            if msg.is_err() {
                let err = msg.unwrap_err();
                match err {
                    Io(e) => return Err(e),
                    _ => continue,
                }
            }
            let deserialized: IncomingMessage =
                serde_json::from_str(&msg.unwrap().to_string()).unwrap();

            match deserialized {
                IncomingMessage::SDPOffer(offer) => {
                    log::info!("Offer Description received : {}", offer.description);
                    let ip = public_ip::get_public_ip().await;

                    self.send_message_to_signal_sever(OutgoingMessage {
                        message_type: OutgoingType::Answer,
                        message: std::format!("{}:{}", ip, server_constants::SERVER_PORT),
                    })
                }
                IncomingMessage::Pong => log::info!("pong"),
            }
        }
    }

    fn send_message_to_signal_sever(&mut self, message: OutgoingMessage) {
        log::info!(
            "type : {} message: {}",
            message.message_type,
            message.message
        );
        match self.socket.write_message(Message::Text(message.message)) {
            Ok(_) => {
                log::info!("Message Written");
            }
            Err(e) => log::error!("Failed to write message : {}", e),
        }
    }
}
