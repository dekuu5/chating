use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::str::FromStr;


pub const USAGE: &str = "\
usage:
  chating setup   --out-dir <dir>
  chating listen  --port <port> --my-priv <path> --peer-pub <path>
  chating connect --ip <ip> --port <port> --my-priv <path> --peer-pub <path>";


#[derive(Debug)]
pub struct Connection {
    mode: Mode,
    ip: Ipv4Addr,
    port: u16,
    my_priv: Option<PathBuf>,
    peer_pub: Option<PathBuf>,
    out_dir: Option<PathBuf>,
}

impl Default for Connection {
    fn default() -> Self {
        Self {
            mode: Mode::Connect,
            ip: Ipv4Addr::new(0, 0, 0, 0),
            port: 0,
            my_priv: None,
            peer_pub: None,
            out_dir: None,
        }
    }
}

impl Connection {
    pub fn mode(&self) -> &Mode { &self.mode }
    pub fn set_mode(&mut self, mode: Mode) { self.mode = mode; }

    pub fn ip(&self) -> Ipv4Addr { self.ip }
    pub fn set_ip(&mut self, ip: Ipv4Addr) { self.ip = ip; }

    pub fn port(&self) -> u16 { self.port }
    pub fn set_port(&mut self, port: u16) { self.port = port; }

    pub fn my_priv(&self) -> Option<&PathBuf> { self.my_priv.as_ref() }
    pub fn set_my_priv(&mut self, p: PathBuf) { self.my_priv = Some(p); }

    pub fn peer_pub(&self) -> Option<&PathBuf> { self.peer_pub.as_ref() }
    pub fn set_peer_pub(&mut self, p: PathBuf) { self.peer_pub = Some(p); }

    pub fn out_dir(&self) -> Option<&PathBuf> { self.out_dir.as_ref() }
    pub fn set_out_dir(&mut self, p: PathBuf) { self.out_dir = Some(p); }
}

#[derive(Debug, Default)]
pub enum Mode {
    #[default]
    Connect,
    Listen,
    Setup,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "listen" => Ok(Mode::Listen),
            "connect" => Ok(Mode::Connect),
            "setup" => Ok(Mode::Setup),
            _ => Err(USAGE.to_string()),
        }
    }
}
