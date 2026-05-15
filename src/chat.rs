use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::types::{Connection, Mode};


impl Connection {
    pub fn run(&self) -> Result<(), String>{
        let mut stream = match &self.mode() {
            Mode::Connect => Self::start_connect(&self)?,
            Mode::Listen => Self::start_listen(&self)?,
        };

        let mut reader = stream.try_clone()
            .map_err(|e| format!("failed to clone the stream {e}"))?;

        let recv_th = std::thread::spawn(move| | {
            let mut buf = [0u8; 1024];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {println!("disconnected"); break ;}
                    Ok(n) => print!("other: {}",String::from_utf8_lossy(&buf[..n])),
                    Err(e) => { eprintln!("recv err {e}"); break;}
                }
            }  
        });

        loop {
            let mut inp = String::new();
            std::io::stdin().read_line(&mut inp)
                .map_err(|e| format!("failed to read from stdin {e}"))?;

            if inp.trim() == "quit" {
                break;

            }

            stream.write_all(inp.as_bytes())
                .map_err(|e| format!("failed to write to streamz {e}"))?;
        }
        
        Ok(())
    }

    fn start_connect(&self) -> Result<TcpStream,String> {
        let stream = TcpStream::connect(
            format!("{}:{}",
            self.ip().to_string(),
            self.port())
        )
        .map_err(|e| format!("failed to connect {e}"))?;

        println!("LOG:Connected to {},{}",
            self.ip().to_string(),
            self.port());
        
        Ok(stream)
        
    }

    fn start_listen(&self) -> Result<TcpStream,String> {
        let listener = TcpListener::bind(
            format!("{}:{}",
            self.ip().to_string(),
            self.port())
        )
        .map_err(|e| format!("failed to bind {e}"))?;

        println!("LOG: Listening on port {}", self.port());
        let (stream, addr) = listener.accept()
            .map_err(|e| format!("failed to accept {e}"))?;

        println!("LOG: connected to {}",addr);

        Ok(stream)
    }

    fn send(stream: &mut TcpStream, msg: &str) -> Result<(),String> {
        stream.write_all(msg.as_bytes())
            .map_err(|e| format!("send failed {e}"))
    }

    fn recv(stream: &mut TcpStream) -> Result<String,String> {
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf)
            .map_err(|e| format!("recv failed: {e}"))?;
        String::from_utf8(buf[..n].to_vec())
            .map_err(|e| format!("invalid utf8: {e}"))
    }
}