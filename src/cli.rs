use std::net::Ipv4Addr;
use std::path::PathBuf;

use crate::types::{Connection, Mode, USAGE};


pub fn parse_cli(chat: &mut Connection) -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        return Err(USAGE.to_string());
    }

    let mode: Mode = args[0].parse()?;

    match mode {
        Mode::Connect => parse_connect(chat, &args[1..])?,
        Mode::Listen => parse_listen(chat, &args[1..])?,
        Mode::Setup => parse_setup(chat, &args[1..])?,
    }

    Ok(())
}

fn take_value<'a>(args: &'a [String], i: usize, name: &str) -> Result<&'a String, String> {
    args.get(i + 1).ok_or_else(|| format!("{name} requires a value"))
}

fn parse_connect(c: &mut Connection, args: &[String]) -> Result<(), String> {
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                let port = take_value(args, i, "--port")?
                    .parse::<u16>()
                    .map_err(|_| "--port must be a valid number")?;
                c.set_port(port);
                i += 2;
            }
            "--ip" => {
                let ip = take_value(args, i, "--ip")?
                    .parse::<Ipv4Addr>()
                    .map_err(|_| "--ip must be a valid ip")?;
                c.set_ip(ip);
                i += 2;
            }
            "--my-priv" => {
                c.set_my_priv(PathBuf::from(take_value(args, i, "--my-priv")?));
                i += 2;
            }
            "--peer-pub" => {
                c.set_peer_pub(PathBuf::from(take_value(args, i, "--peer-pub")?));
                i += 2;
            }
            other => return Err(format!("unknown argument {other}\n{USAGE}")),
        }
    }

    if c.port() == 0 || c.my_priv().is_none() || c.peer_pub().is_none() {
        return Err(format!("connect requires --ip, --port, --my-priv, --peer-pub\n{USAGE}"));
    }

    c.set_mode(Mode::Connect);
    Ok(())
}

fn parse_listen(c: &mut Connection, args: &[String]) -> Result<(), String> {
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                let port = take_value(args, i, "--port")?
                    .parse::<u16>()
                    .map_err(|_| "--port must be a valid number")?;
                c.set_port(port);
                i += 2;
            }
            "--my-priv" => {
                c.set_my_priv(PathBuf::from(take_value(args, i, "--my-priv")?));
                i += 2;
            }
            "--peer-pub" => {
                c.set_peer_pub(PathBuf::from(take_value(args, i, "--peer-pub")?));
                i += 2;
            }
            other => return Err(format!("unknown argument {other}\n{USAGE}")),
        }
    }

    if c.port() == 0 || c.my_priv().is_none() || c.peer_pub().is_none() {
        return Err(format!("listen requires --port, --my-priv, --peer-pub\n{USAGE}"));
    }

    c.set_mode(Mode::Listen);
    c.set_ip(Ipv4Addr::new(0, 0, 0, 0));
    Ok(())
}

fn parse_setup(c: &mut Connection, args: &[String]) -> Result<(), String> {
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out-dir" => {
                c.set_out_dir(PathBuf::from(take_value(args, i, "--out-dir")?));
                i += 2;
            }
            other => return Err(format!("unknown argument {other}\n{USAGE}")),
        }
    }

    if c.out_dir().is_none() {
        return Err(format!("setup requires --out-dir\n{USAGE}"));
    }

    c.set_mode(Mode::Setup);
    Ok(())
}
