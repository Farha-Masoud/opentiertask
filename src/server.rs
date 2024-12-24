extern crate log;
extern crate env_logger;
extern crate prost;

use prost::Message;
use log::{error, info, warn};
use std::io::{self, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

// Define the EchoMessage protocol
#[derive(Clone, PartialEq, prost::Message)]
pub struct EchoMessage {
    #[prost(string, tag = "1")]
    pub content: String,
}

impl EchoMessage {
    pub fn decode(buffer: &[u8]) -> Result<Self, prost::DecodeError> {
        EchoMessage::decode(buffer)
    }
}

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512];
        loop {
            let bytes_read = self.stream.read(&mut buffer)?;
            if bytes_read == 0 {
                info!("Client disconnected.");
                break;
            }

            if let Ok(message) = EchoMessage::decode(&buffer[..bytes_read]) {
                info!("Received: {}", message.content);
                let payload = message.encode_to_vec();
                self.stream.write_all(&payload)?;
                self.stream.flush()?;
            } else {
                error!("Failed to decode message");
            }
        }

        Ok(())
    }
}

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
}

impl Server {
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let is_running = Arc::new(AtomicBool::new(true));
        Ok(Server {
            listener,
            is_running,
        })
    }

    pub fn run(&self) -> io::Result<()> {
        info!("Server is running on {}", self.listener.local_addr()?);
        self.listener.set_nonblocking(true)?;

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);
                    let mut client = Client::new(stream);
                    if let Err(e) = client.handle() {
                        error!("Error handling client: {}", e);
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        info!("Server stopped.");
        Ok(())
    }

    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}

fn main() -> io::Result<()> {
    env_logger::init(); // Initialize logging environment

    let server_addr = "127.0.0.1:8080";
    let server = Server::new(server_addr)?;

    let server_thread = thread::spawn(move || {
        if let Err(e) = server.run() {
            error!("Server error: {}", e);
        }
    });

    // Simulate stopping the server after 10 seconds
    thread::sleep(Duration::from_secs(10));
    server.stop();
    server_thread.join().unwrap();
    Ok(())
}
