use std::io;
use irc::client::prelude::*;
use plugin::Plugin;

register_plugin!(H);

impl<'a> H<'a> {
    fn h(&self, message: &Message, target: &str) -> io::Result<()> {
        let nickname = message.source_nickname().unwrap_or("");
        self.server.send_privmsg(target,
                                 &format!("h {}", nickname))
    }
}

impl<'a> Plugin for H<'a> {
    fn is_allowed(&self, message: &Message) -> bool {
        match message.command {
            Command::PRIVMSG(_, ref msg) => {
                let my_name = self.server.current_nickname();
                msg.trim() == &format!("h {}", my_name)
            },
            _ => false
        }
    }

    fn execute(&mut self, message: &Message) -> io::Result<()> {
        match message.command {
            Command::PRIVMSG(ref target, _) => self.h(message, target),
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
        let server = make_server("PRIVMSG test :h Gauss\r\n");

        for message in server.iter() {
            let     message = message.unwrap();
            let mut plugin  = H::new(&server);

            assert!(plugin.is_allowed(&message));
            assert!(plugin.execute(&message).is_ok());
        }

        assert_eq!("PRIVMSG test :h \r\n", &*get_server_value(server));
    }

    #[test]
    fn test_not_allowed() {
        let server = make_server("PRIVMSG test :h Holo\r\n");

        for message in server.iter() {
            let message = message.unwrap();
            let plugin  = H::new(&server);

            assert!(!plugin.is_allowed(&message));
        }
    }
}
