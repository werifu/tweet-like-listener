use downloader::{Downloader, TIMEOUT};

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
use tokio::{task, time::sleep}; // 1.3.1

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

# usernames you would like to listen, with prefix '@'
usernames = ['@werifu_']

# request frequency. unit: 1 second
freq = 5

[storage]
# the dir that stores images
dir = './pic'
"#;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    // check the network
    tokio::spawn(async {
        loop {
            let res = reqwest::Client::new().get("https://twitter.com").timeout(TIMEOUT).send().await;
            match res {
                Ok(_) => {
                    info!("ping Twitter ok");
                },
                Err(_) => {
                    error!("ping Twitter failed. You may not reach twitter. (tips tor China Mainland users: check your proxy)");
                }
            };
            sleep(Duration::from_secs(10)).await;
        }
    });
    match fs::read_to_string("./config.toml") {
        Ok(config_str) => {
            let config: Config = toml::from_str(&config_str).unwrap();
            let dir = Path::new(config.storage.dir.as_str());
            if !dir.is_dir() {
                info!("{:?} is not a dir", dir);
                exit(0);
            }
            info!("start to get user id by usernames");
            let usernames = config
                .twitter
                .usernames
                .iter()
                .map(|username| &username[1..])
                .collect();
            let mut downloader = Downloader::new(config.twitter.access_key);
            let users = match downloader.get_users_by_usernames(usernames).await {
                Ok(users) => users,
                Err(_) => {
                    error!("You may not reach twitter. Please check your config and net. (tips tor China Mainland users: check your proxy)");
                    exit(1);
                }
            };
            info!("ok users: {:?}", users);

            loop {
                for user in users.iter() {
                    match downloader.get_likes(user.id.clone()).await {
                        Ok(likes) => {
                            info!(
                                "{} get {} likes",
                                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                                likes.len()
                            );
                            let mut handles = vec![];
                            for (filename, url) in likes.iter() {
                                let filename = filename.clone();
                                let url = url.clone();
                                let full_path = dir.join(&filename);
                                if full_path.exists() {
                                    continue;
                                }
                                // download pictures concurrently
                                handles.push(task::spawn(async move {
                                    match reqwest::Client::new().get(url).send().await {
                                        Ok(img_bytes) => {
                                            let img_bytes = img_bytes.bytes().await.unwrap();
                                            let mut f = File::create(full_path).unwrap();
                                            f.write(&img_bytes).unwrap();
                                        }
                                        Err(err) => {
                                            error!("download file {} error: {:?}", filename, err);
                                        }
                                    }
                                }));
                            }
                            // await for all download tasks
                            for handle in handles {
                                let _ = handle.await;
                            }
                        }
                        Err(err) => {
                            error!("error happened, err: {:?}", err);
                        }
                    }
                }
                sleep(Duration::from_secs(config.twitter.freq)).await;
            }
        }
        Err(err) => {
            info!(
                "config file is not found, a new config.toml is to be created. err: {:#?}",
                err
            );
            let mut file = File::create("./config.toml").unwrap();
            file.write_all(CONFIG_CONTENT.as_bytes()).unwrap();
        }
    };
    Ok(())
}
