extern crate hyper;
extern crate serde;
extern crate serde_json;

use std::io::{self, Read};
use irc::client::prelude::*;
use regex::Regex;
use plugin::Plugin;
use hyper::client::Client;
use hyper::header::Connection;
use serde_json::Value;

register_plugin!(Currency);

lazy_static! {
    static ref RE: Regex = Regex::new(r"([0-9]+) ([A-Za-z]+) (?i)(to) ([A-Za-z]+)").unwrap();
}

struct ConvertionRequest<'a> {
    value:  f64,
    source: &'a str,
    target: &'a str
}

macro_rules! try_option {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None    => { return None; }
        }
    }
}

impl<'a> ConvertionRequest<'a> {
    fn send(&self) -> Option<f64> {
        let client   = Client::new();
        let response = client.get(&*format!("http://api.fixer.io/latest?base={}", self.source))
            .header(Connection::close())
            .send();

        match response {
            Ok(mut response) => {
                let mut body = String::new();
                try_option!(response.read_to_string(&mut body).ok());

                let convertion_rates: Result<Value, _> = serde_json::from_str(&body);
                match convertion_rates {
                    Ok(convertion_rates) => {
                        let key = format!("rates.{}", self.target.to_uppercase());

                        let target_rate: &Value = try_option!(convertion_rates.lookup(&*key));
                        Some(self.value * try_option!(target_rate.as_f64()))
                    },
                    Err(_) => None
                }
            },
            Err(_) => None
        }
    }
}

impl Currency {
    fn grep_request<'a>(&self, msg: &'a str) -> Option<ConvertionRequest<'a>> {
        match RE.captures(msg) {
            Some(captures) => Some(ConvertionRequest {
                value:  { let capture = try_option!(captures.at(1)); try_option!(capture.parse().ok()) },
                source: try_option!(captures.at(2)),
                target: try_option!(captures.at(4)) // 3 is to/TO
            }),
            None => None
        }
    }

    fn convert(&self, server: &IrcServer, _: &Message, target: &str, msg: &str) -> io::Result<()> {
        let request = match self.grep_request(msg) {
            Some(request) => request,
            None          => { return Ok(()); }
        };

        match request.send() {
            Some(response) => {
                server.send_privmsg(target, &*format!("{} {} => {:.4} {}",
                                                      request.value, request.source, response / 1.00000000, request.target))
            },
            None => server.send_privmsg(target, "Error while converting given currency")
        }
    }
}

impl Plugin for Currency {
    fn is_allowed(&self, _: &IrcServer, message: &Message) -> bool {
        match message.command {
            Command::PRIVMSG(_, ref msg) => RE.is_match(msg),
            _ => false
        }
    }

    fn execute(&mut self, server: &IrcServer, message: &Message) -> io::Result<()> {
        match message.command {
            Command::PRIVMSG(ref target, ref msg) => self.convert(server, message, target, msg),
            _ => Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use ::tests::{make_server, get_server_value};

    use irc::client::prelude::*;

    use plugin::Plugin;
    use regex::Regex;
    use super::Currency;

    #[test]
    fn test_big_jpy_to_eur() {
        let     server = make_server("PRIVMSG test :5000000 JPY to EUR\r\n");
        let mut plugin = Currency::new();

        for message in server.iter() {
            let message = message.unwrap();
            assert!(plugin.is_allowed(&server, &message));
            assert!(plugin.execute(&server, &message).is_ok());
        }

        let regex = Regex::new(r"=> ([0-9]{2})").unwrap();
        let msg = get_server_value(&server);
        let captures = regex.captures(&*msg).unwrap();
        assert_eq!(captures.at(0), Some("=> 43"));
    }
}
