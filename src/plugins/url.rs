use std::io;
use irc::client::prelude::*;
use regex::Regex;
use plugin::Plugin;

extern crate kuchiki;
use kuchiki::traits::*;

register_plugin!(Url);

lazy_static! {
    static ref RE: Regex = Regex::new(r"http(s)?://(\S+)").unwrap();
}

impl Url {
    fn grep_url(&self, msg: &str) -> Option<String> {
        match RE.captures(msg) {
            Some(captures) => {
                if captures.len() == 3 {
                    Some(format!("https://{}", captures.at(2).unwrap()))
                }
                else {
                    Some(format!("http://{}", captures.at(1).unwrap()))
                }
            },
            None => None
        }
    }

    fn url(&self, server: &IrcServer, _: &Message, target: &str, msg: &str) -> io::Result<()> {
        let url = match self.grep_url(msg) {
            Some(url) => url,
            None      => { return Ok(()); }
        };

        if let Ok(doc) = kuchiki::parse_html().from_http(&url) {
            let matches = doc.select("title").unwrap().last().unwrap();
            let node    = matches.as_node().first_child().unwrap();
            let title   = node.as_text().unwrap().borrow();
            server.send_privmsg(target,
                                &format!("[URL] {}", &*title))
        }
        else {
            Ok(())
        }
    }
}

impl Plugin for Url {
    fn is_allowed(&self, _: &IrcServer, message: &Message) -> bool {
        match message.command {
            Command::PRIVMSG(_, ref msg) => RE.is_match(msg),
            _ => false
        }
    }

    fn execute(&mut self, server: &IrcServer, message: &Message) -> io::Result<()> {
        match message.command {
            Command::PRIVMSG(ref target, ref msg) => self.url(server, message, target, msg),
            _ => Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use ::tests::{make_server, get_server_value};

    use irc::client::prelude::*;

    use plugin::Plugin;
    use super::Url;

    #[test]
    fn test_url() {
        let     server = make_server("PRIVMSG test :https://github.com\r\n");
        let mut plugin = Url::new();

        for message in server.iter() {
            let message = message.unwrap();
            assert!(plugin.is_allowed(&server, &message));
            assert!(plugin.execute(&server, &message).is_ok());
        }

        assert_eq!("PRIVMSG test :[URL] How people build software · GitHub\r\n",
                   &*get_server_value(&server));
    }

    #[test]
    fn test_url_not_given() {
        let server = make_server("PRIVMSG test :httplol\r\n");
        let plugin = Url::new();

        for message in server.iter() {
            let message = message.unwrap();
            assert!(!plugin.is_allowed(&server, &message));
        }
    }
}
