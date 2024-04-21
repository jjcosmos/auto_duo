use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub jwt: String,
    pub driver_path: String,
    pub firefox_exe_path: String,
    pub fallbacks: Vec<Fallback>,
}

#[derive(Serialize, Deserialize)]
pub struct Fallback {
    pub base_url: String,
    pub start_tag: String,
    pub all_matches: bool,
    pub separator: Option<String>,
    pub lang_tag: Option<String>,
}

pub fn read_config() -> Option<Config> {
    // Example config object for users to replicate
    let dummy = Config {
        jwt: String::new(),
        driver_path: String::new(),
        firefox_exe_path: String::new(),
        fallbacks: vec![
            Fallback {
                base_url: "https://jisho.org/search/".to_string(),
                start_tag: r#"<span class="meaning-meaning">"#.to_string(),
                all_matches: false,
                separator: Some(";".to_string()),
                lang_tag: Some("ja".to_string()),
            },
            Fallback {
                base_url: "https://www.lingq.com/en/learn-japanese-online/translate/ja/".to_string(),
                start_tag: "<span class=\"copy-text\">".to_string(),
                all_matches: false,
                separator: Some(",".to_string()),
                lang_tag: Some("ja".to_string()),
            },
        ],
    };

    match fs::File::open("config.json") {
        Ok(mut file) => {
            let mut text = String::new();
            file.read_to_string(&mut text).unwrap();
            let config = serde_json::from_str(&text).unwrap();
            return config;
        }
        Err(_) => {
            let str = serde_json::to_string_pretty(&dummy).unwrap();
            let mut f = fs::File::create("config.json").unwrap();
            f.write_all(str.as_bytes()).unwrap();
            return None;
        }
    }
}
