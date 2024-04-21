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
    pub separator: Option<String>,
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
                separator: Some(";".to_string()),
            },
            Fallback {
                base_url: "https://www.romajidesu.com/translator/".to_string(),
                start_tag: r#"<div class="res_translated" id="res_english">"#.to_string(),
                separator: None,
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
