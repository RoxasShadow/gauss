extern crate irc;
extern crate regex;
#[macro_use] extern crate lazy_static;

#[macro_use] mod plugin;
mod plugins;

use std::default::Default;
use std::io;
use irc::client::prelude::*;

use plugin::Plugin;

fn dispatch(server: &IrcServer, message: Message) -> io::Result<()> {
    let plugins = vec![
        plugins::h::H::new(&server)
    ];

    for plugin in plugins {
        if plugin.is_allowed(&message) {
            try!(plugin.execute(&message));
        }
    }

    Ok(())
}

fn main() {
    let cfg = Config {
        nickname: Some("Gauss".into()),
        server:   Some("irc.rizon.net".into()),
        channels: Some(vec!["#aggvistnurummor".into()]),
        ..Default::default()
    };

    let server = IrcServer::from_config(cfg).unwrap();
    server.identify().unwrap();

    for message in server.iter() {
        let message = message.unwrap();
        dispatch(&server, message).unwrap();
    }
}
