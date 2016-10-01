use std::io;
use irc::client::prelude::*;
use regex::Regex;
use plugin::Plugin;
use time::{self, Tm};

lazy_static! {
    static ref RE: Regex = Regex::new(r"!seen (.+)").unwrap();
}

#[derive(PartialEq, Debug)]
struct User {
    pub name:      String,
    pub joined_at: Tm,
    pub parted_at: Option<Tm>,
}

impl ToString for User {
    fn to_string(&self) -> String {
        match self.parted_at {
            Some(parted_at) => {
                format!("{} joined here on {} and parted on {}",
                        self.name,
                        self.joined_at.rfc822(),
                        parted_at.rfc822())
            },
            None => {
                format!("{} joined here on {}",
                        self.name,
                        self.joined_at.rfc822())
            }
        }
    }
}

register_plugin!(Seen, users: Vec<User>);

impl<'a> Seen<'a> {
    fn grep_username<'b>(&self, msg: &'b str) -> Option<&'b str> {
        match RE.captures(&msg) {
            Some(captures) => captures.at(1),
            None           => None
        }
    }

    fn joined(&mut self, nickname: Option<String>) -> io::Result<()> {
        match nickname {
            Some(nickname) => {
                let user = User {
                    name:      nickname,
                    joined_at: time::now(),
                    parted_at: None
                };

                if !self.users.contains(&user) {
                    self.users.push(user);
                }
            },
            None => {}
        }

        Ok(())
    }

    fn parted(&mut self, nickname: Option<String>) -> io::Result<()> {
        match nickname {
            Some(nickname) => {
                self.users.iter_mut().find(|u| u.name == nickname).map(|mut u| u.parted_at = Some(time::now()));
            },
            None => {}
        }

        Ok(())
    }

    fn seen(&mut self, message: &Message, target: &str, msg: &str) -> io::Result<()> {
        let username = match self.grep_username(msg) {
            Some(user) => user,
            None      => { return Ok(()); }
        };

        if username == self.server.current_nickname() {
            return self.server.send_privmsg(target, "That's me!");
        }

        let requester = message.source_nickname();
        if requester.is_some() && username == requester.unwrap() {
            return self.server.send_privmsg(target, "That's you!");
        }

        for user in &self.users {
            if user.name == username {
                return self.server.send_privmsg(target, &user.to_string());
            }
        }

        self.server.send_privmsg(target,
                                 &format!("I haven't seen {}", username))
    }
}

impl<'a> Plugin for Seen<'a> {
    fn is_allowed(&self, _: &Message) -> bool {
        true
    }

    fn execute(&mut self, message: &Message) -> io::Result<()> {
        match message.command {
            Command::PRIVMSG(ref target, ref msg) => self.seen(message, target, msg),
            Command::JOIN(_, _, _)                => self.joined(message.source_nickname().map(|n| n.to_owned())),
            Command::PART(_, _)                   => self.parted(message.source_nickname().map(|n| n.to_owned())),
            _ => Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use ::tests::{make_server, get_server_value};

    use irc::client::prelude::*;

    use plugin::Plugin;
    use time::{self, Tm};
    use super::{Seen, User};

    fn get_time() -> Tm {
        time::strptime("Sat, 01 Oct 2016 12:58:34 GMT", "%a, %d %b %Y %T GMT").unwrap()
    }

    #[test]
    fn test_seen() {
        let     server  = make_server("PRIVMSG test :!seen Holo\r\n");
        let mut plugin = Seen::new(&server);
        plugin.users.push(User { name: "Gauss".to_owned(), joined_at: get_time(), parted_at: None });
        plugin.users.push(User { name: "Holo".to_owned(),  joined_at: get_time(), parted_at: Some(get_time()) });

        let message = server.iter().last().unwrap().unwrap();
        assert!(plugin.is_allowed(&message));
        assert!(plugin.execute(&message).is_ok());
        assert_eq!(plugin.users.len(), 2);

        let users: Vec<String> = plugin.users.iter().map(|u| u.name.to_owned()).collect();
        assert_eq!(users, vec!["Gauss".to_owned(), "Holo".to_owned()]);

        assert_eq!("PRIVMSG test :Holo joined here on Sat, 01 Oct 2016 12:58:34 GMT and parted on Sat, 01 Oct 2016 12:58:34 GMT\r\n",
            &*get_server_value(&server));
    }

    #[test]
    fn test_its_me() {
        let     server = make_server("PRIVMSG test :!seen Gauss\r\n");
        let mut plugin = Seen::new(&server);
        plugin.users.push(User { name: "Gauss".to_owned(), joined_at: get_time(), parted_at: None });

        let message = server.iter().last().unwrap().unwrap();
        assert!(plugin.is_allowed(&message));
        assert!(plugin.execute(&message).is_ok());
        assert_eq!(plugin.users.len(), 1);

        assert_eq!("PRIVMSG test :That's me!\r\n", &*get_server_value(&server));
    }

    #[test]
    fn test_not_seen() {
        let     server = make_server("PRIVMSG test :!seen Holo\r\n");
        let mut plugin = Seen::new(&server);
        plugin.users.push(User { name: "Gauss".to_owned(), joined_at: get_time(), parted_at: None });

        let message = server.iter().last().unwrap().unwrap();
        assert!(plugin.is_allowed(&message));
        assert!(plugin.execute(&message).is_ok());
        assert_eq!(plugin.users.len(), 1);

        assert_eq!("PRIVMSG test :I haven't seen Holo\r\n", &*get_server_value(&server));
    }
}
