use std::process::exit;

use openssl::rsa::Rsa;

use crate::{cli::parse_cli, types::Connection};

mod cli;
mod types;
mod chat;
mod crypto;


fn main() {
    
    let mut chat = Connection::default();
    
    match parse_cli(&mut chat) {
        Ok(()) => {},
        Err(s) => {
            println!("{s}");
            exit(-1);
        }
    };

    match chat.run(){
        Err(s) => {
            println!("{s}");
            exit(-1);
        }
        _ => {}
    }
    println!("{:?}",chat);
    
}
