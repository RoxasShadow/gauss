extern crate irc;
extern crate regex;
extern crate kuchiki;
extern crate time;
#[macro_use] extern crate lazy_static;

#[macro_use] mod plugin;
mod plugins;

use std::default::Default;
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

    let mut plugins: Vec<Box<Plugin>> = vec![
        Box::new(plugins::h::H::new(&server)),
        Box::new(plugins::url::Url::new(&server)),
        Box::new(plugins::seen::Seen::new(&server)),
    ];

    for message in server.iter() {
        let message = message.unwrap();

        for mut plugin in plugins.iter_mut() {
            if plugin.is_allowed(&message) {
                plugin.execute(&message).unwrap();
            }
        }
    }
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
