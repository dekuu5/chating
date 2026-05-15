
use std::net::Ipv4Addr;

use crate::types::{Connection, Mode, USAGE};






pub fn parse_cli(chat :&mut Chat) -> Result<(),String> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        return Err("Usage: chating <listen|connect> [options]".to_string());
    }
    
    let mode: Mode = args[0].parse()?;

    match mode {
        Mode::Connect => parse_connect(chat, &args[1..])?,
        Mode::Listen => parse_listen(chat, &args[1..])?,
    }

    Ok(())
}

fn parse_connect(c :&mut Chat,args:  &[String]) -> Result<(), String> {
    let mut i = 0;
    if args.len() < 4{
        return Err(format!("{}",USAGE));
    }
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                let port = args
                    .get(i+1)
                    .ok_or("--port require a value")?
                    .parse::<u16>()
                    .map_err(|_| "--port must be a valid number")?;
                c.set_port(port);
                i = i+2;
            },
            "--ip" => {
                let ip = args
                    .get(i+1)
                    .ok_or("--ip require a value")?
                    .parse::<Ipv4Addr>()
                    .map_err(|_| "--ip must be a valid ip")?;
                c.set_ip(ip);
                i = i+2;
            }
            other => return Err(format!("unknown Arguments {other}\n{}",USAGE)),
        }
    }
    
    c.set_mode(Mode::Connect);
    Ok(())
}
fn parse_listen(c :&mut Chat, args:  &[String]) -> Result<(), String> {
    let mut i = 0;
    if args.len() < 2{
        return Err(format!("{}",USAGE));
    }
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                let port = args
                    .get(i+1)
                    .ok_or("--port require a value")?
                    .parse::<u16>()
                    .map_err(|_| "--port must be a valid number")?;
                c.set_port(port);
                i = i+2
            },
            other => return Err(format!("unknown Arguments {other}\n{}",USAGE)),
        }
    }
    
    c.set_mode(Mode::Listen);
    c.set_ip(Ipv4Addr::new(0, 0, 0, 0));
    Ok(())
}