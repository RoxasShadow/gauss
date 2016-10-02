use std::io;
use std::env;
use std::sync::Mutex;
use irc::client::prelude::*;
use regex::Regex;
use rustfm::*;
use plugin::Plugin;

lazy_static! {
    static ref RE: Regex = Regex::new(r"!addlastfmuser (.+)").unwrap();
}

#[derive(PartialEq, Debug, Clone)]
struct LastFMUser {
    irc_username:    String,
    lastfm_username: String
}

register_plugin!(LastFM, users: Vec<LastFMUser>);

impl LastFM {
    fn grep_username<'a>(&self, msg: &'a str) -> Option<&'a str> {
        match RE.captures(&msg) {
            Some(captures) => captures.at(1),
            None           => None
        }
    }

    fn add_user(&mut self, server: &IrcServer, message: &Message, target: &str, msg: &str) -> io::Result<()> {
        match message.source_nickname() {
            Some(nickname) => match self.grep_username(msg) {
                Some(lastfm_username) => {
                    let user = LastFMUser {
                        irc_username:    nickname.to_owned(),
                        lastfm_username: lastfm_username.to_owned()
                    };

                    if let Some(index) = self.users.iter().position(|u| u.irc_username == nickname) {
                        self.users.remove(index);
                    }
                    else {
                        if let Some(index) = self.users.iter().position(|u| u.irc_username == nickname) {
                            self.users.remove(index);
                            self.users.push(user.clone());
                        }
                    }

                    self.users.push(user.clone());

                    server.send_privmsg(target,
                                        &*format!("{} is now associated to the LastFM user {}", user.irc_username, user.lastfm_username))
                },
                None => Ok(())
            },
            None => Ok(())
        }
    }

    fn lastsong(&self, server: &IrcServer, message: &Message, target: &str) -> io::Result<()> {
        lazy_static! {
            static ref CLIENT: Mutex<Client> = match env::var("LASTFM_API_KEY") {
                Ok(api_key) => Mutex::new(Client::new(&*api_key)),
                Err(e)      => { panic!("No LASTFM_API_KEY found (error: {})", e); }
            };
        }


        match message.source_nickname() {
            Some(nickname) => {
                let username = if let Some(user) = self.users.iter().find(|u| u.irc_username == nickname) {
                    user.lastfm_username.to_owned()
                }
                else {
                    nickname.to_owned()
                };

                match CLIENT.lock().unwrap().recent_tracks(&*username).with_limit(1).send() {
                    Ok(recent_tracks) => match recent_tracks.tracks.first() {
                        Some(track) => server.send_privmsg(target,
                                                           &*format!("The last song {} listened to is {} by {} (in {}){})",
                                                           username,
                                                           track.name,
                                                           track.artist,
                                                           track.album,
                                                           if let Some(ref date) = track.date { format!(", on {}", date) } else { String::new() })),
                        None => server.send_privmsg(target, &*format!("I don't know what is the last song {} listened to. Try !addlastfmuser", nickname))
                    },
                    Err(e) => server.send_privmsg(target, &*format!("Something bad happened: {:?}", e))
                }
            },
            None => Ok(())
        }
    }
}


impl Plugin for LastFM {
    fn is_allowed(&self, _: &IrcServer, message: &Message) -> bool {
        match message.command {
            Command::PRIVMSG(_, ref msg) => msg == "!lastsong" || self.grep_username(msg).is_some(),
            _ => false
        }
    }

    fn execute(&mut self, server: &IrcServer, message: &Message) -> io::Result<()> {
        match message.command {
            Command::PRIVMSG(ref target, ref msg) => {
                if msg == "!lastsong" {
                    self.lastsong(server, message, target)
                }
                else if self.grep_username(msg).is_some() {
                    self.add_user(server, message, target, msg)
                }
                else {
                    Ok(())
                }
            },
            _ => Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use ::tests::make_server;

    use irc::client::prelude::*;

    use plugin::Plugin;
    use super::LastFM;

    #[test]
    fn test_lastsong() {
        let     server = make_server("PRIVMSG test :!lastsong\r\n");
        let mut plugin = LastFM::new();

        for message in server.iter() {
            let message = message.unwrap();
            assert!(plugin.is_allowed(&server, &message));
            assert!(plugin.execute(&server, &message).is_ok());
            // source_nickname() is None, this won't work
        }
    }

    #[test]
    fn test_add_user() {
        let     server = make_server("PRIVMSG test :!addlastfmuser Gaussimandro\r\n");
        let mut plugin = LastFM::new();

        for message in server.iter() {
            let message = message.unwrap();
            assert!(plugin.is_allowed(&server, &message));
            assert!(plugin.execute(&server, &message).is_ok());
            // source_nickname() is None, this won't work
        }
    }
}
