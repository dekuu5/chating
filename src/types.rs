use std::{net::Ipv4Addr, str::FromStr};



pub const USAGE: &str = "usage: \n chating listen --port port \n  chating connect --ip ip --port port";


#[derive(Debug)]
pub struct Connection {
    mode: Mode,
    ip : Ipv4Addr,
    port: u16,
}

impl Default for Connection {
    fn default() -> Self {
        Self { 
            mode: Mode::Connect,
            ip: Ipv4Addr::new(0, 0, 0, 0),
            port: 0 
        }
    }
}

impl Connection {
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn ip(&self) -> Ipv4Addr {
        self.ip
    }

    pub fn set_ip(&mut self, ip: Ipv4Addr) {
        self.ip = ip;
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }
}

#[derive(Debug,Default)]
pub enum Mode {
    #[default]
    Connect,
    Listen
}



impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "listen" =>  Ok(Mode::Listen),
            "connect" => Ok(Mode::Connect),
            _ => Err(format!("{}",USAGE)),
        }
    }
}