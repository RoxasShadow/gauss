extern crate irc;
extern crate regex;
extern crate kuchiki;
#[macro_use] extern crate lazy_static;

#[macro_use] mod plugin;
mod plugins;

use std::default::Default;
use std::io;
use irc::client::prelude::*;

use plugin::Plugin;

fn dispatch(server: &IrcServer, message: Message) -> io::Result<()> {
    let plugins: Vec<Box<Plugin>> = vec![
        Box::new(plugins::h::H::new(&server)),
        Box::new(plugins::url::Url::new(&server)),
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

#[cfg(test)]
mod tests {
    use irc::client::prelude::*;
    use irc::client::conn::MockConnection;

    pub fn make_server(cmd: &str) -> IrcServer {
        let connection = MockConnection::new(cmd);

        let config = Config {
            nickname: Some("Gauss".into()),
            server:   Some("irc.test.net".into()),
            channels: Some(vec!["#test".into()]),
            ..Default::default()
        };

        IrcServer::from_connection(config, connection)
    }

    pub fn get_server_value(server: IrcServer) -> String {
        server.conn().written(server.config().encoding()).unwrap()
    }
}
