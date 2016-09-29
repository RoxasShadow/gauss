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

    fn execute(&self, message: &Message) -> io::Result<()> {
        match message.command {
            Command::PRIVMSG(ref target, _) => self.h(message, target),
            _ => Ok(())
        }
    }
}
