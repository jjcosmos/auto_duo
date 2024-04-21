#[cfg(test)]
mod tests {
    use crate::*;
    use serial_test::serial;

    const START_DELAY: u64 = 2;

    #[tokio::test]
    #[serial]
    async fn test_spanish() {
        sleep(Duration::from_secs(START_DELAY)).await; // Make sure port is free from any previous runs. Hacky, but fine for tests.

        let config = config::read_config().unwrap();
        let gecko = Command::new("cmd")
            .args(&["/C", "start", &config.driver_path])
            .spawn()
            .unwrap();

        let _guard = CrashGuard(gecko);

        let mut caps = DesiredCapabilities::firefox();
        caps.set_firefox_binary(&config.firefox_exe_path).unwrap();
        let driver = WebDriver::new("http://localhost:4444", caps).await.unwrap();

        nav_to_lang(&driver, &config, "Spanish").await.unwrap();
        duo_read(&driver, &config).await.unwrap();
        driver.quit().await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_japanese() {
        sleep(Duration::from_secs(START_DELAY)).await;

        let config = config::read_config().unwrap();
        let gecko = Command::new("cmd")
            .args(&["/C", "start", &config.driver_path])
            .spawn()
            .unwrap();

        let _guard = CrashGuard(gecko);

        let mut caps = DesiredCapabilities::firefox();
        caps.set_firefox_binary(&config.firefox_exe_path).unwrap();
        let driver = WebDriver::new("http://localhost:4444", caps).await.unwrap();

        nav_to_lang(&driver, &config, "Japanese").await.unwrap();
        duo_read(&driver, &config).await.unwrap();
        driver.quit().await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_german() {
        sleep(Duration::from_secs(START_DELAY)).await;

        let config = config::read_config().unwrap();
        let gecko = Command::new("cmd")
            .args(&["/C", "start", &config.driver_path])
            .spawn()
            .unwrap();

        let _guard = CrashGuard(gecko);

        let mut caps = DesiredCapabilities::firefox();
        caps.set_firefox_binary(&config.firefox_exe_path).unwrap();
        let driver = WebDriver::new("http://localhost:4444", caps).await.unwrap();

        nav_to_lang(&driver, &config, "German").await.unwrap();
        duo_read(&driver, &config).await.unwrap();
        driver.quit().await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn change_language_test() {
        sleep(Duration::from_secs(START_DELAY)).await;

        let config = config::read_config().unwrap();
        let gecko = Command::new("cmd")
            .args(&["/C", "start", &config.driver_path])
            .spawn()
            .unwrap();

        let _guard = CrashGuard(gecko);

        let mut caps = DesiredCapabilities::firefox();
        caps.set_firefox_binary(&config.firefox_exe_path).unwrap();
        let driver = WebDriver::new("http://localhost:4444", caps).await.unwrap();

        nav_to_lang(&driver, &config, "Spanish").await.unwrap();
        nav_to_lang(&driver, &config, "Japanese").await.unwrap();
        driver.quit().await.unwrap();
    }
}
