use config::Config;
use reqwest::Error;

use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{BufRead, BufReader},
    process::{Child, Command},
    time::Duration,
};
use thirtyfour::{prelude::*, support::sleep};

use crate::config::Browser;
mod config;
mod tests;

struct CrashGuard(Child);

impl Drop for CrashGuard {
    fn drop(&mut self) {
        match self.0.kill() {
            Ok(_) => {
                println!("Killed child process on drop")
            }
            Err(e) => {
                eprintln!("Could not kill child process on drop: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let config = config::read_config().unwrap();

    let gecko = Command::new("cmd")
        .args(&["/C", "start", &config.driver_path])
        .spawn()
        .expect(format!("Could not spaw driver proc from {}", &config.driver_path).as_str());

    let id = gecko.id();
    println!("Started driver with PID {}", id);

    // Connect to the webdriver
    let driver = match config.browser {
        Browser::Firefox => {
            let mut caps = DesiredCapabilities::firefox();
            if config.headless {caps.set_headless().unwrap();}
            caps.set_firefox_binary(&config.firefox_exe_path).unwrap();
            WebDriver::new("http://localhost:4444", caps).await?
        },
        Browser::Chrome => {
            let mut caps = DesiredCapabilities::chrome();
            if config.headless {caps.set_headless().unwrap();}
            caps.set_binary(&config.firefox_exe_path).unwrap();
            WebDriver::new("http://localhost:9515", caps).await?

        },
    };

    let _guard = CrashGuard(gecko);

    // Do the actual duo
    match duo_read(&driver, &config).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e)
        }
    };

    // Close everything
    driver.quit().await?;

    Ok(())
}

fn build_dict_from_txt() -> HashMap<String, Vec<String>> {
    let file = fs::File::open("words.txt").unwrap();
    let reader = BufReader::new(file);

    let mut map = HashMap::new();

    let mut is_key = true;
    let mut last_key = String::new();
    for line in reader.lines() {
        let text = line.unwrap_or_default();
        let trimmed = text.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if is_key {
            last_key = trimmed.to_string();
            map.insert(last_key.clone(), Vec::<String>::new());
        } else {
            let split = trimmed.split(",");
            for s in split {
                let trimmed_split = s.trim();
                map.get_mut(&last_key)
                    .unwrap()
                    .push(trimmed_split.to_owned());
            }
        }

        is_key = !is_key;
    }

    map
}

async fn duo_read(driver: &WebDriver, config: &Config) -> WebDriverResult<()> {
    let text_dict = build_dict_from_txt();

    let secret_key = "jwt_token";
    let secret_value = &config.jwt;

    driver.goto("https://duolingo.com/learn").await?;
    driver
        .add_cookie(Cookie::new(secret_key, secret_value))
        .await?;

    driver.refresh().await?;
    driver.goto("https://www.duolingo.com/practice-hub").await?;
    driver.enter_default_frame().await?;

    sleep(Duration::from_secs(5)).await;

    let long_path = false;
    if long_path {
        let elem: WebElement = driver.find(By::Css("button._1eJKW:nth-child(2)")).await?;
        println!("{:?}", elem.id().await?);
        elem.click().await?;

        driver.enter_default_frame().await?;

        let elem = driver.find(By::Css(".-TADL")).await?;
        elem.click().await?;

        sleep(Duration::from_secs(5)).await;
    }

    driver
        .goto("https://www.duolingo.com/practice-hub/words/practice")
        .await?;
    sleep(Duration::from_secs(5)).await; // TODO: Better wait

    driver.enter_default_frame().await?;
    let mut button_confirm = driver
        .query(By::Tag("button"))
        .with_attribute("data-test", "player-next")
        .first()
        .await?;
    button_confirm.click().await?;

    sleep(Duration::from_secs(1)).await;

    query_and_click_set(&driver, &text_dict, 2, config).await?;

    // Click confirm
    button_confirm.click().await?;

    // Click the can't listen button
    sleep(Duration::from_secs(2)).await;
    let cant_listen = driver
        .query(By::Tag("button"))
        .with_attribute("data-test", "player-skip")
        .first()
        .await?;
    cant_listen.click().await?;

    // Click continue onto next matches
    sleep(Duration::from_secs(1)).await;
    button_confirm.click().await?;

    sleep(Duration::from_secs(2)).await;

    let mut tries = 0;
    let max_tries = 30;

    //let footer = driver.find(By::Id("session/PlayerFooter")).await?;

    loop {
        //let correct_q = driver.find(By::Css("._1BBhb")).await.is_ok();
        let correct_q = button_confirm
            .attr("aria-disabled")
            .await?
            .unwrap_or_default()
            == "false";

        if tries > max_tries {
            return Err(WebDriverError::FatalError(
                "Exceeded max tries.".to_string(),
            ));
        }

        if correct_q {
            println!("Found continue button. Assuming done.");
            sleep(Duration::from_secs(1)).await;
            if !button_confirm.is_present().await? {
                button_confirm = driver
                    .query(By::Tag("button"))
                    .with_attribute("data-test", "player-next")
                    .first()
                    .await?;
            }
            println!(
                "Clicking {}...",
                button_confirm.text().await.unwrap_or("NONE".to_owned())
            );
            button_confirm.click().await?;
            break;
        } else {
            // Long delay here, as the answers regenerate slowly sometimes
            // TODO: broke this lol. try a run with empty txt to smooth out runs
            // Also romajidesu gives weird output...
            query_and_click_set(&driver, &text_dict, 0, config).await?;
            tries += 1;
        }
    }

    sleep(Duration::from_secs(2)).await;
    if !button_confirm.is_present().await? {
        button_confirm = driver
            .query(By::Tag("button"))
            .with_attribute("data-test", "player-next")
            .first()
            .await?;
    }

    println!(
        "Clicking {}...",
        button_confirm.text().await.unwrap_or("NONE".to_owned())
    );
    button_confirm.click().await?;

    sleep(Duration::from_secs(1)).await;

    // Spam confirm button
    loop {
        if driver.current_url().await?.to_string().ends_with("words") {
            // Made it back home, stop spamming
            break;
        }

        if !button_confirm.is_present().await? {
            button_confirm = driver
                .query(By::Tag("button"))
                .with_attribute("data-test", "player-next")
                .first()
                .await?;
        }

        println!(
            "Clicking {}...",
            button_confirm.text().await.unwrap_or("NONE".to_owned())
        );
        button_confirm.click().await?;
        sleep(Duration::from_secs(1)).await;
    }

    println!("Done! Shutting down...");
    sleep(Duration::from_secs(2)).await;

    Ok(())
}

#[allow(dead_code)]
async fn nav_to_lang(
    driver: &WebDriver,
    config: &Config,
    lang_full_name: &str,
) -> WebDriverResult<()> {
    let secret_key = "jwt_token";
    let secret_value = &config.jwt;

    driver.goto("https://duolingo.com/learn").await?;
    driver
        .add_cookie(Cookie::new(secret_key, secret_value))
        .await?;

    driver.refresh().await?;
    driver.goto("https://www.duolingo.com/practice-hub").await?;
    driver.enter_default_frame().await?;

    sleep(Duration::from_secs(5)).await;

    let lang_select = driver.find(By::Css("._3TvKV")).await?;
    driver
        .action_chain()
        .move_to_element_center(&lang_select)
        .perform()
        .await?;

    sleep(Duration::from_secs(1)).await;

    let lang_dropdown_buttons = lang_select.find_all(By::ClassName("_3oF3u")).await?;
    let mut success = false;
    for button in &lang_dropdown_buttons {
        let text = button.text().await?.to_lowercase();
        if text == lang_full_name.to_lowercase() {
            println!("Found element");
            button.click().await?;
            success = true;
            sleep(Duration::from_secs(2)).await;
            break;
        }
    }

    if !success {
        let mut texts = vec![];
        for canditate in &lang_dropdown_buttons {
            let text = canditate.text().await.unwrap_or("NONE".to_string());
            texts.push(text);
        }
        let err_txt = format!(
            "Could not find match for {} in {:?}",
            &lang_full_name, texts
        );
        return Err(WebDriverError::FatalError(err_txt));
    }

    sleep(Duration::from_secs(3)).await;
    driver.action_chain().reset_actions().await?;

    Ok(())
}

async fn query_and_click_set(
    driver: &WebDriver,
    text_dict: &HashMap<String, Vec<String>>,
    end_delay_seconds: u64,
    config: &Config,
) -> WebDriverResult<()> {
    // do a button click pass
    let mut english = vec![];
    let mut japanese = vec![];

    let mut buttons_en = vec![];
    let mut buttons_jp = vec![];

    let query = driver
        .query(By::ClassName("notranslate"))
        .with_attribute("aria-disabled", "false");
    let mut lang_tag = String::new();

    for q in query.any().await? {
        if !q.is_present().await? {
            continue;
        }

        let child = q
            .find(By::Css("span:nth-child(3) > span:nth-child(1)"))
            .await?;

        if !child.is_present().await? {
            continue;
        }

        let text = child.text().await?;

        let tag = q.attr("lang").await?.unwrap_or_default();
        let is_en = tag == "en".to_string();
        println!("Text: {} tag: {}", text, tag);

        if !is_en {
            japanese.push(text);
            buttons_jp.push(q);
            lang_tag = tag;
        } else {
            english.push(text);
            buttons_en.push(q);
        }
    }

    //assert_eq!(english.len(), japanese.len());

    for jp_tl in &japanese {
        let res = get_match_multi(&jp_tl, &english, &text_dict, &config, &lang_tag).await?;
        match res {
            Some(en_tl) => {
                println!("Found match for target lang: {} en: {}", jp_tl, en_tl);
                let index_of_en = english
                    .iter()
                    .position(|en_str| *en_str == en_tl)
                    .expect("Couldn't find en TL in vector?");
                let index_of_jp = japanese
                    .iter()
                    .position(|jp_str| jp_str == jp_tl)
                    .expect("Couldn't find jp TL in vector?");

                let button_en = buttons_en.get(index_of_en).unwrap();
                let button_jp = buttons_jp.get(index_of_jp).unwrap();

                button_en.click().await?;
                sleep(Duration::from_millis(200)).await;
                button_jp.click().await?;
                sleep(Duration::from_millis(200)).await;
            }
            None => {
                eprintln!("Found no matches for {}!", &jp_tl);
                continue;
            }
        }
    }

    if japanese.len() == 0 {
        println!("Found no results, trying again.");
        return Ok(());
    } else {
        // Wait for new answers to regenerate
        println!("Waiting for new batch of pairs ({}sec)", end_delay_seconds);
        sleep(Duration::from_secs(end_delay_seconds)).await;
        return Ok(());
    }
}

async fn get_match_multi(
    jp: &str,
    en_options: &Vec<String>,
    text_dict: &HashMap<String, Vec<String>>,
    config: &Config,
    lang_tag: &str,
) -> Result<Option<String>, Error> {
    // First, check if it is in the wordlist provided in words.txt.
    // This is the fastest and most reliable, especially in languages with the standart character set
    match text_dict.get(jp) {
        Some(vec) => {
            let lookup_set: HashSet<&String> = HashSet::from_iter(vec.iter());
            let options_set: HashSet<&String> = HashSet::from_iter(en_options.iter());

            let intersection: Vec<&String> =
                lookup_set.intersection(&options_set).map(|f| *f).collect();

            if intersection.len() == 1 {
                return Ok(Some((*intersection.first().unwrap()).to_owned()));
            } else {
                eprintln!(
                    "Set intersection found none or too many choices! {:?}",
                    &intersection
                );
            }
        }
        None => {}
    }

    // Next, iterate through all the fallbacks provided in the config in order, returning on the first success
    for fallback in &config.fallbacks {
        // Check if this fallback is exclusive to a language
        match &fallback.lang_tag {
            Some(tag) => {
                if tag != lang_tag {
                    continue;
                }
            }
            None => {}
        }

        match get_match_extended(
            en_options,
            &fallback.base_url,
            jp,
            &fallback.start_tag,
            fallback.separator.clone().as_deref(),
            fallback.all_matches,
        )
        .await
        {
            Ok(success) => {
                // We can still get none if there was no fatal error
                // In that case, just move to next fallback.
                if success.is_some() {
                    return Ok(success);
                }
            }
            Err(_) => {}
        }
    }

    // Found nothing, but didn't error
    Ok(None)
}

async fn get_match_extended(
    en_options: &Vec<String>,
    site_root: &str,
    search_term: &str,
    start_tag: &str,
    split_pattern: Option<&str>,
    all_matches: bool,
) -> Result<Option<String>, Error> {
    eprintln!("Falling back to {} for {}...", &site_root, &search_term);

    let frmt = format!("{}{}", site_root, search_term);
    let response = reqwest::get(&frmt).await?.text().await?;

    let t;
    let potential_found = response.find(start_tag);
    match potential_found {
        Some(found) => t = found,
        None => {
            eprintln!(
                "Did not find tag {} from url {}. {} bytes checked",
                &start_tag,
                &frmt,
                response.as_bytes().len()
            );
            return Ok(None);
        } // Maybe not a valid url, but not necessarily the end
    };

    let start_byte = t + start_tag.bytes().len();
    //let as_bytes = response.as_bytes();
    let mut byte_index = start_byte;

    loop {
        let end_byte = byte_index + response[start_byte..].find("<").unwrap();
        let slice = &response[byte_index..end_byte];

        byte_index += slice.len();

        let splits = match split_pattern {
            Some(patt) => slice.split(patt).collect(),
            None => vec![slice],
        };

        for s in &splits {
            let trimmed = s.trim().to_lowercase();

            for en in en_options {
                let mut lower = en.trim().to_lowercase();

                // remove plural from english button option. TODO: kinda jank
                if lower.ends_with("s") {
                    lower = lower[0..lower.len() - 1].to_string();
                }

                if trimmed.contains(&lower) {
                    return Ok(Some(en.to_owned()));
                }
            }
        }

        let easy: Vec<&str> = splits.into_iter().map(|s| s).collect();
        eprintln!(
            "Splits do not match! target: {} scraped: {:?} options: {:?}",
            search_term, easy, en_options
        );

        // In the event of matching tags with garbage data, just call it quits.
        if !all_matches {
            break;
        }

        // Otherwise, move byte index to next occurence
        // Important to remember that find is the index in the SLICE, not the source collection...
        match response[byte_index..].find(start_tag) {
            Some(found_index) => {
                byte_index = byte_index + found_index + start_tag.bytes().len();
            }
            None => {
                break;
            }
        }
    }

    return Ok(None);
}
