use std::io;
use irc::client::prelude::*;
use regex::Regex;
use plugin::Plugin;

extern crate kuchiki;
use kuchiki::traits::*;

register_plugin!(Tangorin);

lazy_static! {
    static ref RE: Regex = Regex::new(r"!tangorin (\S+)").unwrap();
}

macro_rules! try_option {
    ( $( $maybe_text: expr);* ) => {
       match $($maybe_text)* {
         Some(text) => text,
         None => { return Ok(()); }
       };
    }
}

impl Tangorin {
    fn grep_kanji(&self, msg: &str) -> Option<String> {
        match RE.captures(msg) {
            Some(captures) => captures.at(1).map(|e| e.to_owned()),
            None => None
        }
    }

    fn retrieve_from_selector(&self, doc: &kuchiki::NodeRef, selector: &str) -> Option<String> {
        doc.select(selector).unwrap().next().map(|match_| {
            let node = match_.as_node().first_child().unwrap();
            let borrowed_text = node.as_text().unwrap().borrow();
            borrowed_text.to_owned().trim().to_string()
        })
    }

    fn retrieve_meaning(&self, doc: &kuchiki::NodeRef) -> Option<String> {
        doc.select("span[class=eng]").unwrap().next().map(|match_| {
            let node_children = match_.as_node().children();
            let mut meaning = String::new();
            for child in node_children {
                if let Some(text) = child.as_text() {
                    meaning.push_str(text.borrow().as_str());
                }
                else if let Some(text) = self.inner_text(&child) {
                    meaning.push_str(text.as_str());
                }
            }

            meaning
        })
    }

    fn retrieve_info(&self, doc: &kuchiki::NodeRef) -> Option<String> {
        match doc.select("span[class=eng]").unwrap().next() {
            None => None,
            Some(match_) => {
                let node = match_.as_node();
                let following_siblings = node.following_siblings();

                match following_siblings.select("i[class=d-info]") {
                    Err(_) => None,
                    Ok(mut info_sibs) => info_sibs.next().map(|first_sibling|
                        self.inner_text(first_sibling.as_node()).unwrap()
                    )
                }
            }
        }
    }

    fn inner_text(&self, root: &kuchiki::NodeRef) -> Option<String> {
        match root.first_child() {
            None => None,
            Some(match_) => {
                match match_.as_text() {
                    Some(result_str) => Some(result_str.borrow().to_owned()),
                    None => None
                }
            }
        }
    }

    fn tangorin(&self, server: &IrcServer, _: &Message, target: &str, msg: &str) -> io::Result<()> {
        let url = match self.grep_kanji(msg) {
            Some(kanji) => format!("http://tangorin.com/general/{}", kanji),
            None      => { return Ok(()); }
        };

        if let Ok(doc) = kuchiki::parse_html().from_http(&url) {
            let romaji = try_option!(self.retrieve_from_selector(&doc, "rt"));
            let kana = try_option!(self.retrieve_from_selector(&doc, "rb"));
            let kanji = try_option!(self.retrieve_from_selector(&doc, "span[class=writing]"));
            let meaning = try_option!(self.retrieve_meaning(&doc));
            let info: String = match self.retrieve_info(&doc) {
                Some(retrieved) => format!(" ({})", retrieved.replace("\u{2014}", "").replace(".", "").to_lowercase()),
                None => String::new()
            };

            return server.send_privmsg(target, &format!("[Tangorin] {} ({} - {}): {}{}", &*kanji, &*kana, &*romaji, &*meaning, &*info))
        }

        Ok(())
    }
}

impl Plugin for Tangorin {
    fn is_allowed(&self, _: &IrcServer, message: &Message) -> bool {
        match message.command {
            Command::PRIVMSG(_, ref msg) => RE.is_match(msg),
            _ => false
        }
    }

    fn execute(&mut self, server: &IrcServer, message: &Message) -> io::Result<()> {
        match message.command {
            Command::PRIVMSG(ref target, ref msg) => self.tangorin(server, message, target, msg),
            _ => Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use ::tests::{make_server, get_server_value};

    use irc::client::prelude::*;

    use plugin::Plugin;
    use super::Tangorin;

    #[test]
    fn test_tangorin() {
        let     server = make_server("PRIVMSG test :!tangorin 桜\r\n");
        let mut plugin = Tangorin::new();

        for message in server.iter() {
            let message = message.unwrap();
            assert!(plugin.is_allowed(&server, &message));
            assert!(plugin.execute(&server, &message).is_ok());
        }

        assert_eq!("PRIVMSG test :[Tangorin] 桜 (さくら - sakura): cherry tree;  cherry blossom\r\n",
                   &*get_server_value(&server));
    }

    #[test]
    fn test_tangorin_write_explanation() {
        let     server = make_server("PRIVMSG test :!tangorin 頑\r\n");
        let mut plugin = Tangorin::new();

        for message in server.iter() {
            let message = message.unwrap();
            assert!(plugin.is_allowed(&server, &message));
            assert!(plugin.execute(&server, &message).is_ok());
        }
        assert_eq!("PRIVMSG test :[Tangorin] 頑な (かたくな - katakuna): obstinate (usually written using kana alone)\r\n",
                   &*get_server_value(&server));
    }

    #[test]
    fn test_tangorin_missing_argument() {
        let server = make_server("PRIVMSG test :!tangorin            \r\n");
        let plugin = Tangorin::new();

        for message in server.iter() {
            let message = message.unwrap();
            assert!(!plugin.is_allowed(&server, &message));
        }
    }

    #[test]
    fn test_tangorin_not_called() {
        let server = make_server("PRIVMSG test :httplol\r\n");
        let plugin = Tangorin::new();

        for message in server.iter() {
            let message = message.unwrap();
            assert!(!plugin.is_allowed(&server, &message));
        }
    }
}
