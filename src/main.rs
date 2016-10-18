#![feature(proc_macro)]
extern crate irc;
extern crate regex;
extern crate kuchiki;
extern crate time;
extern crate rustfm;
extern crate serde;
extern crate hyper;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;

#[macro_use] mod plugin;
mod plugins;

use std::env;
use std::default::Default;
use std::thread::spawn;
use std::sync::{Arc, Mutex};
use irc::client::prelude::*;

use plugin::Plugin;

fn main() {
    let mut args = env::args();

    let exe_name = args.next().unwrap();
    if args.len() < 3 {
        panic!("Usage: {} [nickname] [server] [\"#channel1\" \"#channel2\"...]", exe_name);
    }

    let config = Config {
        nickname: args.next(),
        server:   args.next(),
        channels: Some(args.collect::<Vec<String>>()),
        ..Default::default()
    };

    let server = IrcServer::from_config(config).unwrap();
    server.identify().unwrap();

    let plugins: Vec<Arc<Mutex<Plugin>>> = vec![
        Arc::new(Mutex::new(plugins::h::H::new())),
        Arc::new(Mutex::new(plugins::url::Url::new())),
        Arc::new(Mutex::new(plugins::seen::Seen::new())),
        Arc::new(Mutex::new(plugins::lastfm::LastFM::new())),
        Arc::new(Mutex::new(plugins::tangorin::Tangorin::new())),
        Arc::new(Mutex::new(plugins::currency::Currency::new())),
    ];

    for message in server.iter() {
        let message = Arc::new(message.unwrap());

        for plugin in plugins.clone().into_iter() {
            let server  = server.clone();
            let message = message.clone();

            spawn(move || {
                let mut plugin = match plugin.lock() {
                    Ok(plugin)    => plugin,
                    Err(poisoned) => poisoned.into_inner()
                };

                if plugin.is_allowed(&server, &message) {
                    plugin.execute(&server, &message).unwrap();
                }
            });
        }
    }

    loop {}
}

#[cfg(test)]
mod tests {
    use irc::client::prelude::*;
    use irc::client::conn::MockConnection;

    pub fn make_server(cmd: &str) -> IrcServer {
        let config = Config {
            nickname: Some("Gauss".into()),
            server:   Some("irc.test.net".into()),
            channels: Some(vec!["#test".into()]),
            ..Default::default()
        };

        let connection = MockConnection::new(cmd);
        IrcServer::from_connection(config, connection)
    }

    pub fn get_server_value(server: &IrcServer) -> String {
        server.conn().written(server.config().encoding()).unwrap()
    }
}
