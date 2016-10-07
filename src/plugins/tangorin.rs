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
    fn grep_url(&self, msg: &str) -> Option<String> {
        match RE.captures(msg) {
            Some(captures) => {
                if captures.len() == 3 {
                    Some(format!("{}", captures.at(2).unwrap()))
                }
                else {
                    Some(format!("{}", captures.at(1).unwrap()))
                }
            },
            None => None
        }
    }

    fn retrieve_romaji(&self, doc: &kuchiki::NodeRef) -> Option<String> {
        if let Some(match_) = doc.select("rt").unwrap().next() {
            let node    = match_.as_node().first_child().unwrap();
            let borrowed_romaji   = node.as_text().unwrap().borrow();
            let mut romaji = borrowed_romaji.clone();
            romaji = romaji.trim().to_string();
            
            Some(romaji)
        }
        else{
            None
        }
    }
    
    fn retrieve_kana(&self, doc: &kuchiki::NodeRef) -> Option<String> {
        if let Some(match_) = doc.select("rb").unwrap().next() {
            let node    = match_.as_node().first_child().unwrap();
            let borrowed_kana   = node.as_text().unwrap().borrow();
            let mut kana = borrowed_kana.clone();
            kana = kana.trim().to_string();
            
            Some(kana)
        }
        else{
            None
        }
    }

    fn retrieve_kanji(&self, doc: &kuchiki::NodeRef) -> Option<String> {
        if let Some(match_) = doc.select("span[class=writing]").unwrap().next() {
            let node = match_.as_node().first_child().unwrap();
            let borrowed_kanji = node.as_text().unwrap().borrow();
            let mut realkanji = borrowed_kanji.clone();
            realkanji = realkanji.trim().to_string();

            Some(realkanji)
        }
        else {
            None
        }
    }
    
    fn retrieve_meaning(&self, doc: &kuchiki::NodeRef) -> Option<String> {
        if let Some(match_) = doc.select("span[class=eng]").unwrap().next() {
            let node = match_.as_node(); 
            let children = node.children();
            let mut meaning = "".to_string();
            for child_ in children {
                if let Some(text_) = child_.as_text() {
                    meaning.push_str(text_.borrow().as_str());
                }
                else{
                    if let Some(text_) = self.deeper_text(&child_) {
                        meaning.push_str(text_.as_str());
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
            let mut following_siblings = node.following_siblings();

            if let Ok(mut info_sibs) = following_siblings.select("i[class=d-info]") {
                match info_sibs.next() {
                    Some(first_sibling) => self.deeper_text(first_sibling.as_node()),
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

    fn deeper_text(&self, root: &kuchiki::NodeRef) -> Option<String> {
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
        let url = match self.grep_url(msg) {
            Some(kanji) => "http://tangorin.com/general/".to_string() + kanji.as_str(),
            None      => { return Ok(()); }
        };

        if let Ok(doc) = kuchiki::parse_html().from_http(&url) {
            let kanji : String = match self.grep_url(msg){
                Some(k) => k,
                None => { return Ok(()); }
            };
          
            let romaji = match self.retrieve_romaji(&doc) {
                Some(retrieved) => retrieved,
                None => { return Ok(()); } 
            };
            
            let kana = match self.retrieve_kana(&doc) {
                Some(retrieved) => retrieved,
                None => { return Ok(()); } 
            };
         
            let retrieved_kanji = match self.retrieve_kanji(&doc) {
                Some(retrieved) => retrieved,
                None => { return Ok(()); }
            };
         
            let meaning = match self.retrieve_meaning(&doc) {
                Some(retrieved) => retrieved,
                None => { return Ok(()); }
            };

            let info : String = match self.retrieve_info(&doc) {
                Some(retrieved) => {
                    let sanitized = retrieved.replace("\u{2014}", "").replace(".", "").to_lowercase();
                    (" (".to_string()+sanitized.as_str()+")").to_string()
                },
                None => "".to_string()
            };
          
            return server.send_privmsg(target, &format!("[Tangorin] {} ({} - {}): {}{}", &*retrieved_kanji, &*kana, &*romaji, &*meaning, &*info))
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
        let     server = make_server("PRIVMSG test :!tangorin            \r\n");
        let mut plugin = Tangorin::new();

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
