use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use crate::crypto::{AESKeys, PrivKey, PubKey, generate_keypair, hash_sha256, random_nonce};
use crate::types::{Connection, Mode};


impl Connection {
    pub fn run(&self) -> Result<(), String> {
        match self.mode() {
            Mode::Setup => self.run_setup(),
            Mode::Connect => self.run_peer(true),
            Mode::Listen => self.run_peer(false),
        }
    }

    fn run_setup(&self) -> Result<(), String> {
        let out = self.out_dir().ok_or("setup: missing --out-dir")?;
        let (my_priv, my_pub) = generate_keypair(out, "my_key")?;
        println!("generated keypair :");
        println!("  private: {}", my_priv.display());
        println!("  public:  {}", my_pub.display());
        println!("\nshare your side's _pub.pem with the other party out-of-band.");
        Ok(())
    }

    fn run_peer(&self, is_connector: bool) -> Result<(), String> {
        let my_priv = PrivKey::load(self.my_priv().ok_or("missing --my-priv")?)?;
        let peer_pub = PubKey::load(self.peer_pub().ok_or("missing --peer-pub")?)?;

        let mut stream = if is_connector {
            self.start_connect()?
        } else {
            self.start_listen()?
        };

        let aes = if is_connector {
            connector_handshake(&mut stream, &my_priv, &peer_pub)?
        } else {
            listener_handshake(&mut stream, &my_priv, &peer_pub)?
        };
        println!("LOG: secure channel established");

        run_chat_loop(stream, aes, my_priv, peer_pub)
    }

    fn start_connect(&self) -> Result<TcpStream, String> {
        let stream = TcpStream::connect(format!("{}:{}", self.ip(), self.port()))
            .map_err(|e| format!("failed to connect {e}"))?;
        println!("LOG: Connected to {}:{}", self.ip(), self.port());
        Ok(stream)
    }

    fn start_listen(&self) -> Result<TcpStream, String> {
        let listener = TcpListener::bind(format!("{}:{}", self.ip(), self.port()))
            .map_err(|e| format!("failed to bind {e}"))?;
        println!("LOG: Listening on port {}", self.port());
        let (stream, addr) = listener.accept()
            .map_err(|e| format!("failed to accept {e}"))?;
        println!("LOG: connected to {}", addr);
        Ok(stream)
    }
}


fn write_frame(stream: &mut TcpStream, data: &[u8]) -> Result<(), String> {
    let len = (data.len() as u32).to_be_bytes();
    stream.write_all(&len).map_err(|e| format!("write len failed: {e}"))?;
    stream.write_all(data).map_err(|e| format!("write data failed: {e}"))?;
    Ok(())
}

fn read_frame(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).map_err(|e| format!("read len failed: {e}"))?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > 16 * 1024 {
        return Err(format!("frame too large: {len}"));
    }
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).map_err(|e| format!("read data failed: {e}"))?;
    Ok(buf)
}


fn connector_handshake(
    stream: &mut TcpStream,
    my_priv: &PrivKey,
    peer_pub: &PubKey,
) -> Result<AESKeys, String> {
    let aes = AESKeys::new();
    let nonce = random_nonce();
    let nonce_hash = hash_sha256(&nonce)?;

    let enc_aes = peer_pub.encrypt(&aes.to_bytes())?;
    let enc_nonce = aes.encrypt(&nonce)?;
    let sig = my_priv.sign(&nonce_hash)?;

    write_frame(stream, &enc_aes)?;
    write_frame(stream, &enc_nonce)?;
    write_frame(stream, &sig)?;

    let resp_enc_nonce = read_frame(stream)?;
    let resp_sig = read_frame(stream)?;
    let resp_nonce = aes.decrypt(&resp_enc_nonce)?;

    if resp_nonce != nonce {
        return Err("handshake: peer returned wrong nonce".to_string());
    }
    let resp_hash = hash_sha256(&resp_nonce)?;
    if !peer_pub.verify(&resp_hash, &resp_sig)? {
        return Err("handshake: peer signature invalid".to_string());
    }
    Ok(aes)
}

fn listener_handshake(
    stream: &mut TcpStream,
    my_priv: &PrivKey,
    peer_pub: &PubKey,
) -> Result<AESKeys, String> {
    let enc_aes = read_frame(stream)?;
    let enc_nonce = read_frame(stream)?;
    let sig = read_frame(stream)?;

    let aes_bytes = my_priv.decrypt(&enc_aes)?;
    let aes = AESKeys::from_bytes(&aes_bytes)?;
    let nonce = aes.decrypt(&enc_nonce)?;
    let nonce_hash = hash_sha256(&nonce)?;

    if !peer_pub.verify(&nonce_hash, &sig)? {
        return Err("handshake: peer signature invalid".to_string());
    }

    let resp_enc_nonce = aes.encrypt(&nonce)?;
    let resp_sig = my_priv.sign(&nonce_hash)?;
    write_frame(stream, &resp_enc_nonce)?;
    write_frame(stream, &resp_sig)?;

    Ok(aes)
}


fn run_chat_loop(
    stream: TcpStream,
    aes: AESKeys,
    my_priv: PrivKey,
    peer_pub: PubKey,
) -> Result<(), String> {
    let aes = Arc::new(aes);
    let peer_pub = Arc::new(peer_pub);

    let mut reader = stream.try_clone()
        .map_err(|e| format!("failed to clone the stream {e}"))?;
    let mut writer = stream;

    let recv_aes = Arc::clone(&aes);
    let recv_pub = Arc::clone(&peer_pub);

    std::thread::spawn(move || {
        loop {
            let cipher = match read_frame(&mut reader) {
                Ok(b) => b,
                Err(e) => { eprintln!("recv err: {e}"); break; }
            };
            let sig = match read_frame(&mut reader) {
                Ok(b) => b,
                Err(e) => { eprintln!("recv err: {e}"); break; }
            };
            match recv_pub.verify(&cipher, &sig) {
                Ok(true) => {}
                Ok(false) => { eprintln!("WARN: signature mismatch, message rejected"); continue; }
                Err(e) => { eprintln!("verify error: {e}"); continue; }
            }
            match recv_aes.decrypt(&cipher) {
                Ok(plain) => print!("OTHER: {}", String::from_utf8_lossy(&plain)),
                Err(e) => eprintln!("decrypt error: {e}"),
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
        if inp.trim() == "" {
            continue;
        }

        let cipher = aes.encrypt(inp.as_bytes())?;
        let sig = my_priv.sign(&cipher)?;
        write_frame(&mut writer, &cipher)?;
        write_frame(&mut writer, &sig)?;
    }

    Ok(())
}
