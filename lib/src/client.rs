use crate::*;
use tungstenite::{connect, Message, stream::MaybeTlsStream};
use url::Url;
use clap::Parser;

pub type WebSocket = tungstenite::WebSocket<MaybeTlsStream<std::net::TcpStream>>;


pub struct Log {
    socket : WebSocket,
}

impl Log {
    pub fn new(port: u16) -> Self {
        let url = format!("ws://localhost:{}/log", port);
        let url = Url::parse(&url).unwrap();
        let (socket, _) = connect(url).expect("Can't connect to the game server");
        Self {
            socket,
        }
    }

    pub fn send(&mut self, message: &str) {
        let message = ClientMessage::Log(message.to_string());
        let message = serde_json::to_string(&message).expect("Error converting message to string");
        self.socket.send(Message::Text(message)).expect("Error writing message");
    }
}

pub trait Runnable {
    fn initialize(&mut self, log: &mut Log);
    fn take_action(&mut self, info: ClientInfo, log: &mut Log) -> Action;
    fn game_over(&self, info: ClientInfo, results: GameResults);
}

#[derive(Parser, Debug)]
pub struct Args {
    /// The port to connect to
    #[arg(short, long)]
    port: u16,

}

/// The protocol for communication and running the bot between the client and 
/// the server. Sends logs and actions to the server when appropriate. 
pub fn run_bot<B : Runnable + Default>() {
    let args = Args::parse();
    let port = args.port;

    let url = format!("ws://localhost:{}/game", port);
    let url = Url::parse(&url).unwrap();
    let (mut game_socket, _) = connect(url).expect("Can't connect to the game server");

    // Give the server a chance to start up
    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut log = Log::new(port);

    let mut bot = B::default();
    bot.initialize(&mut log);
    println!("Connected to the game server...");

    loop {
        let msg = game_socket.read().expect("Error reading message");
        let msg = msg.to_text().expect("Error converting message to text");
        let info: ClientInfo = serde_json::from_str(msg).expect("Error parsing message");
        let action = bot.take_action(info, &mut log);
        let msg = ClientMessage::Action(action);

        let msg_str = serde_json::to_string(&msg).expect("Error converting action to string");
        game_socket.send(Message::Text(msg_str)).expect("Error sending message");

    }
}