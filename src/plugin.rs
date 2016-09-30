use std::io;
use irc::client::prelude::*;

pub trait Plugin {
    fn is_allowed(&self, Message: &Message) -> bool;
    fn execute(&self, Message: &Message)    -> io::Result<()>;
}

#[macro_export]
macro_rules! register_plugin {
    ($t:ident) => {
        pub struct $t<'a> {
            server: &'a IrcServer
        }

        impl<'a> $t<'a> {
            pub fn new(server: &'a IrcServer) -> $t {
                $t { server: server }
            }
        }
    }
}