use super::websocket::{
    messaging::{answer_generator::AnswerGenerator, pinger_job::PingerJob},
    socket_manager::SocketManager,
};
use crate::service::websocket::signal_connection_maker;

pub struct WebService {}

pub async fn init() {
    let answer_generator: AnswerGenerator = AnswerGenerator {
        answer_service: "string".to_owned(),
    };
    let pinger_job: PingerJob = PingerJob::new(None);

    let connection_maker = signal_connection_maker::SignalConnectionMaker {};
    let mut socket_manager: SocketManager = SocketManager::new(connection_maker);

    socket_manager
        .set_answer_generator(answer_generator)
        .set_pinger_job(pinger_job);

    tokio::spawn(async move {
        socket_manager.listen().await;
    });
}
