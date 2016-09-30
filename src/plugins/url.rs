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

impl<'a> Url<'a> {
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

    fn url(&self, _: &Message, target: &str, msg: &str) -> io::Result<()> {
        let url = match self.grep_url(msg) {
            Some(url) => url,
            None      => { return Ok(()); }
        };

        if let Ok(doc) = kuchiki::parse_html().from_http(&url) {
            let matches = doc.select("title").unwrap().last().unwrap();
            let node    = matches.as_node().first_child().unwrap();
            let title   = node.as_text().unwrap().borrow();
            self.server.send_privmsg(target,
                                     &format!("[URL] {}", &*title))
        }
        else {
            Ok(())
        }
    }
}

impl<'a> Plugin for Url<'a> {
    fn is_allowed(&self, message: &Message) -> bool {
        match message.command {
            Command::PRIVMSG(_, ref msg) => RE.is_match(msg),
            _ => false
        }
    }

    fn execute(&self, message: &Message) -> io::Result<()> {
        match message.command {
            Command::PRIVMSG(ref target, ref msg) => self.url(message, target, msg),
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
        let server = make_server("PRIVMSG test :https://github.com\r\n");

        for message in server.iter() {
            let message = message.unwrap();
            let plugin  = Url::new(&server);

            assert!(plugin.is_allowed(&message));
            assert!(plugin.execute(&message).is_ok());
        }

        assert_eq!("PRIVMSG test :[URL] How people build software · GitHub\r\n", &*get_server_value(server));
    }

    #[test]
    fn test_url_not_found() {
        let server = make_server("PRIVMSG test :https://lolwut.vbb\r\n");

        for message in server.iter() {
            let message = message.unwrap();
            let plugin  = Url::new(&server);

            assert!(plugin.is_allowed(&message));
            assert!(plugin.execute(&message).is_ok());
        }

        assert_eq!("", &*get_server_value(server));
    }

    #[test]
    fn test_url_not_given() {
        let server = make_server("PRIVMSG test :httplol\r\n");

        for message in server.iter() {
            let message = message.unwrap();
            let plugin  = Url::new(&server);

            assert!(!plugin.is_allowed(&message));
        }
    }
}
