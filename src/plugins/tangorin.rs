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

impl Tangorin {
    fn grep_kanji(&self, msg: &str) -> Option<String> {
        match RE.captures(msg) {
            Some(captures) => captures.at(1).map(|e| e.to_owned()),
            None => None
        }
    }

    fn retrieve_from_selector(&self, doc: &kuchiki::NodeRef, selector: &str) -> Option<String> {
        if let Some(match_) = doc.select(selector).unwrap().next() {
            let node = match_.as_node().first_child().unwrap();
            let borrowed_text = node.as_text().unwrap().borrow();
            let retrieved_text = borrowed_text.clone().trim().to_string();
            
            Some(retrieved_text)
        }
        else{
            None
        }
    }

    fn retrieve_meaning(&self, doc: &kuchiki::NodeRef) -> Option<String> {
        if let Some(match_) = doc.select("span[class=eng]").unwrap().next() {
            let node = match_.as_node(); 
            let children = node.children();
            let mut meaning = String::new();
            for child in children {
                if let Some(text) = child.as_text() {
                    meaning.push_str(text.borrow().as_str());
                }
                else{
                    if let Some(text) = self.inner_text(&child) {
                        meaning.push_str(text.as_str());
                    }
                }
            }

           Some(meaning)
        }
        else{
            None
        }
    }

    fn retrieve_info(&self, doc: &kuchiki::NodeRef) -> Option<String> {
        if let Some(match_) = doc.select("span[class=eng]").unwrap().next() {
            let node = match_.as_node();
            let following_siblings = node.following_siblings();

            if let Ok(mut info_sibs) = following_siblings.select("i[class=d-info]") {
                match info_sibs.next() {
                    Some(first_sibling) => self.inner_text(first_sibling.as_node()),
                    None => None
                }
            }
            else{
                None
            }
        }
        else {
            None
        }
    }

    fn inner_text(&self, root: &kuchiki::NodeRef) -> Option<String> {
        if let Some(match_) = root.first_child() {
            let res = match match_.as_text() {
                Some(result_str) => result_str.borrow(),
                None => { return None }
            };
            Some(res.clone())
        }
        else {
            None
        }
    }

    fn tangorin(&self, server: &IrcServer, _: &Message, target: &str, msg: &str) -> io::Result<()> {
        let url = match self.grep_kanji(msg) {
            Some(kanji) => format!("http://tangorin.com/general/{}", kanji),
            None      => { return Ok(()); }
        };

        if let Ok(doc) = kuchiki::parse_html().from_http(&url) {
            let romaji = match self.retrieve_from_selector(&doc, "rt"){
                Some(retrieved) => retrieved,
                None => { return Ok(()); } 
            };
            
            let kana = match self.retrieve_from_selector(&doc, "rb") {
                Some(retrieved) => retrieved,
                None => { return Ok(()); } 
            };
         
            let kanji = match self.retrieve_from_selector(&doc, "span[class=writing]") {
                Some(retrieved) => retrieved,
                None => { return Ok(()); }
            };
         
            let meaning = match self.retrieve_meaning(&doc) {
                Some(retrieved) => retrieved,
                None => { return Ok(()); }
            };

            let info : String = match self.retrieve_info(&doc) {
                Some(retrieved) => {
                    format!(" ({})", retrieved.replace("\u{2014}", "").replace(".", "").to_lowercase())
                },
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
