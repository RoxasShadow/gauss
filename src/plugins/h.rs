use std::io;
use irc::client::prelude::*;
use plugin::Plugin;

register_plugin!(H);

impl H {
    fn h(&self, server: &IrcServer, message: &Message, target: &str) -> io::Result<()> {
        let nickname = message.source_nickname().unwrap_or("");
        server.send_privmsg(target,
                            &format!("h {}", nickname))
    }
}

impl Plugin for H {
    fn is_allowed(&self, server: &IrcServer, message: &Message) -> bool {
        match message.command {
            Command::PRIVMSG(_, ref msg) => {
                let my_name = server.current_nickname();
                msg.trim() == &format!("h {}", my_name)
            },
            _ => false
        }
    }

    fn execute(&mut self, server: &IrcServer, message: &Message) -> io::Result<()> {
        match message.command {
            Command::PRIVMSG(ref target, _) => self.h(server, message, target),
            _ => Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use ::tests::{make_server, get_server_value};

    use irc::client::prelude::*;

    use plugin::Plugin;
    use super::H;

    #[test]
    fn test_allowed() {
        let     server = make_server("PRIVMSG test :h Gauss\r\n");
        let mut plugin  = H::new();

        for message in server.iter() {
            let message = message.unwrap();
            assert!(plugin.is_allowed(&server, &message));
            assert!(plugin.execute(&server, &message).is_ok());
        }

        assert_eq!("PRIVMSG test :h \r\n", &*get_server_value(&server));
    }

    #[test]
    fn test_not_allowed() {
        let server = make_server("PRIVMSG test :h Holo\r\n");
        let plugin  = H::new();

        for message in server.iter() {
            let message = message.unwrap();
            assert!(!plugin.is_allowed(&server, &message));
        }
    }
}
