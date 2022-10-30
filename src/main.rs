use downloader::Downloader;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

use log::{error, info};
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process::exit,
    time::Duration,
};
use tokio::time::sleep; // 1.3.1

mod downloader;
mod model;
mod url;

#[derive(Deserialize, Debug)]
struct TwitterConfig {
    access_key: String,
    usernames: Vec<String>,
    freq: u64,
}

#[derive(Deserialize, Debug)]
struct StorageConfig {
    dir: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    twitter: TwitterConfig,
    storage: StorageConfig,
}

const CONFIG_CONTENT: &str = r#"
[twitter]
# access_key of your application
access_key = 'your twitter access_key'

# usernames you would like to listen
usernames = ['@werifu_']

# request frequency. unit: 1 second
freq = 5

[storage]
dir = './pic'
"#;


#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    match fs::read_to_string("./config.toml") {
        Ok(config_str) => {
            let config: Config = toml::from_str(&config_str).unwrap();
            let dir = Path::new(config.storage.dir.as_str());
            if !dir.is_dir() {
                info!("{:?} is not a dir", dir);
                exit(0);
            }
            let usernames = config
                .twitter
                .usernames
                .iter()
                .map(|username| &username[1..])
                .collect();
            let mut downloader = Downloader::new(config.twitter.access_key);
            let users = match downloader.get_users_by_usernames(usernames).await {
                Ok(users) => users,
                Err(err) => {
                    error!("usernames maybe wrong.\nerr: {:#?}", err);
                    exit(1);
                }
            };

            loop {
                for user in users.iter() {
                    let likes = downloader.get_likes(user.id.clone()).await.unwrap();
                    println!("get likes: {:#?}", likes);
                    for (filename, url) in likes.iter() {
                        let full_path = dir.join(filename);
                        if full_path.exists() {
                            println!("file {:?} exists", full_path);
                            continue;
                        }
                        let img_bytes = reqwest::Client::new()
                            .get(url)
                            .send()
                            .await
                            .unwrap()
                            .bytes()
                            .await
                            .unwrap();
                        println!("fullpath: {:?}", full_path);
                        let mut f = File::create(full_path).unwrap();
                        f.write(&img_bytes).unwrap();
                    }
                }
                sleep(Duration::from_secs(config.twitter.freq)).await;
            }
        }
        Err(err) => {
            println!("config file is not found, {:#?}", err);
            let mut file = File::create("./config.toml").unwrap();
            file.write_all(CONFIG_CONTENT.as_bytes()).unwrap();
        }
    };
    Ok(())
}
