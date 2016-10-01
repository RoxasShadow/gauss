use std::io;
use std::fmt;
use irc::client::prelude::*;

pub trait Plugin: Send + Sync + fmt::Debug {
    fn is_allowed(&self, server: &IrcServer, Message: &Message)  -> bool;
    fn execute(&mut self, server: &IrcServer, Message: &Message) -> io::Result<()>;
}

#[macro_export]
macro_rules! register_plugin {
    ($t:ident) => {
        #[derive(Debug)]
        pub struct $t;

        impl $t {
            pub fn new() -> $t {
                $t { }
            }
        }
    };

    ($t:ident, $element: ident: $ty: ty) => {
        #[derive(Debug)]
        pub struct $t {
            $element: $ty
        }

        impl $t {
            pub fn new() -> $t {
                $t { $element: <$ty>::new() }
            }
        }
    };
}
