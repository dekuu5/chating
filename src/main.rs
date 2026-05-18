use std::process::exit;

use crate::{cli::parse_cli, types::Connection};

mod cli;
mod types;
mod chat;
mod crypto;


fn main() {
    let mut chat = Connection::default();

    if let Err(s) = parse_cli(&mut chat) {
        eprintln!("{s}");
        exit(1);
    }

    if let Err(s) = chat.run() {
        eprintln!("{s}");
        exit(1);
    }
}
