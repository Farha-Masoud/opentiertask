use log::{error, info};
use prost::Message;
use std::io::{Read, Write};
use std::net::{TcpStream, Shutdown};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Define the EchoMessage protocol
#[derive(Clone, PartialEq, prost::Message)]
pub struct EchoMessage {
    #[prost(string, tag = "1")]
    pub content: String,
}

pub struct Server {
    stream: Option<TcpStream>,
    is_running: bool,
    listener: TcpListener,
}

impl Server {
    pub fn new(addr: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(Server {
            stream: None,
            is_running: false,
            listener,
        })
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        self.is_running = true;
        info!("Server is running on {}", self.listener.local_addr()?);

        while self.is_running {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    info!("New client connected");
                    self.stream = Some(stream);
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }

    pub fn stop(&mut self) -> std::io::Result<()> {
        self.is_running = false;
        if let Some(stream) = &self.stream {
            stream.shutdown(Shutdown::Both)?;
        }
        info!("Server stopped.");
        Ok(())
    }
}

// Define the Client
pub struct Client {
    server: Arc<Mutex<Server>>,
    timeout: Duration,
}

impl Client {
    pub fn new(server: Arc<Mutex<Server>>, timeout_ms: u64) -> Self {
        Client {
            server,
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    pub fn connect(&self) -> std::io::Result<()> {
        let mut server = self.server.lock().unwrap(); // Lock the server
        server.run()?; // Ensure the server is running
        Ok(())
    }

    pub fn disconnect(&self) -> std::io::Result<()> {
        let mut server = self.server.lock().unwrap(); // Lock the server
        server.stop()?; // Stop the server
        Ok(())
    }

    pub fn send(&self, message: &EchoMessage) -> std::io::Result<()> {
        let mut server = self.server.lock().unwrap(); // Lock the server
        if let Some(ref mut stream) = server.stream {
            let payload = message.encode_to_vec(); // Encode the message
            stream.write_all(&payload)?;
            stream.flush()?;
            info!("Sent message: {:?}", message.content);
            Ok(())
        } else {
            error!("No active connection");
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "No active connection",
            ))
        }
    }

    pub fn receive(&self) -> std::io::Result<EchoMessage> {
        let mut server = self.server.lock().unwrap(); // Lock the server
        if let Some(ref mut stream) = server.stream {
            info!("Receiving message from the server");
            let mut buffer = vec![0u8; 512];
            let bytes_read = stream.read(&mut buffer)?;

            if bytes_read == 0 {
                info!("Server disconnected.");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "Server disconnected",
                ));
            }

            info!("Received {} bytes from the server", bytes_read);
            let message = EchoMessage::decode(&buffer[..bytes_read])?;
            Ok(message)
        } else {
            error!("No active connection");
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "No active connection",
            ))
        }
    }
}
