extern crate irc;
extern crate regex;
extern crate kuchiki;
extern crate time;
#[macro_use] extern crate lazy_static;

#[macro_use] mod plugin;
mod plugins;

use std::default::Default;
use std::thread::spawn;
use std::sync::{Arc, Mutex};
use irc::client::prelude::*;

use plugin::Plugin;

fn main() {
    let cfg = Config {
        nickname: Some("Gauss".into()),
        server:   Some("irc.rizon.net".into()),
        channels: Some(vec!["#aggvistnurummor".into()]),
        ..Default::default()
    };

    let server = IrcServer::from_config(cfg).unwrap();
    server.identify().unwrap();

    let plugins: Vec<Arc<Mutex<Plugin>>> = vec![
        Arc::new(Mutex::new(plugins::h::H::new())),
        Arc::new(Mutex::new(plugins::url::Url::new())),
        Arc::new(Mutex::new(plugins::seen::Seen::new()))
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
