use log::{debug, error, info};
use tungstenite::{connect, Message, client};
use url::Url;
use std::str;

pub fn connect_to_signaling_original() {
let url = Url::parse("wss://signal-service-m7vo.onrender.com/connect/v1/mediaServer/tatatest").unwrap();
let j =  client::connect_with_config(url, None, 1);
// connect_with_config
let (mut socket, response) =
        j.expect("Can't connect");
        

// str::from_utf8(response.body())
    info!("Connected to the server");
    info!("Response HTTP code: {}", response.status());
    info!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        info!("* {}", header);
    }

    loop {
        debug!("Waiting for message");
        let msg = socket.read_message().unwrap();
        debug!("Received: {}", msg);
        socket
            .write_message(Message::Text(msg.to_string()))
            .unwrap();
    }

    // loop {
    //     let msg = socket.read_message().expect("Error reading message");
    //     info!("Received: {}", msg);
    // }
    // socket.close(None);
}