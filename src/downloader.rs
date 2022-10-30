use std::collections::HashMap;
use std::process::exit;

use hyper::http::HeaderValue;
use hyper::{HeaderMap, StatusCode};

use crate::model::{Attachments, Media, Tweet, TweetResp, User, UserResp};
use crate::url::UrlBuilder;
use crate::Result;
use log::{error, debug};
pub struct Downloader {
    pub user_cache: HashMap<String, User>,
    pub user_ids: Vec<String>,
    access_key: String,
}

impl Downloader {
    pub fn new(access_key: String) -> Self {
        Downloader {
            user_cache: HashMap::new(),
            user_ids: vec![],
            access_key,
        }
    }

    /// Get HTTP Authorization header
    fn tweet_auth_header(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let bearer = "Bearer ".to_string() + &self.access_key;
        headers.insert(
            "Authorization",
            HeaderValue::from_str(bearer.as_str()).unwrap(),
        );
        headers
    }

    /// Get a list of img filenames and urls
    pub async fn get_likes(&mut self, user_id: String) -> Result<Vec<(String, String)>> {
        let mut result: Vec<(String, String)> = vec![];
        let client = reqwest::Client::new();
        let url = UrlBuilder::new(
            format!("https://api.twitter.com/2/users/{}/liked_tweets", user_id).as_str(),
        )
        .param("expansions", "attachments.media_keys")
        .param("media.fields", "url")
        .param("tweet.fields", "created_at,author_id")
        .param("max_results", "50")
        .get_url();
        let headers = self.tweet_auth_header();

        let resp = client.get(url).headers(headers).send().await?;
        if resp.status() == StatusCode::UNAUTHORIZED {
            error!("401: Maybe you have a wrong access_key");
            exit(1);
        }

        let resp = resp.json::<TweetResp>().await?;

        let mut media_map: HashMap<String, Media> = HashMap::new();
        if let Some(includes) = resp.includes {
            for media in includes.media {
                let key = media.media_key.clone();
                media_map.insert(key, media);
            }
        }

        if let Some(mut data) = resp.data {
            // cache the authors
            let mut uncached_ids: Vec<&str> = vec![];
            for tweet in data.iter() {
                if !self.user_cache.contains_key(&tweet.author_id) {
                    uncached_ids.push(tweet.author_id.as_str());
                }
            }
            if uncached_ids.len() > 0 {
                match self.get_users_by_ids(uncached_ids).await {
                    Ok(users) => {
                        users.iter().for_each(|user| {
                            self.user_cache.insert(user.id.clone(), user.to_owned());
                        });
                    }
                    Err(err) => {
                        error!("failed to get users because: {}", err.to_string());
                    }
                }
            }

            // assembly needed info
            for tweet in data.iter_mut() {
                // get user info
                if self.user_cache.contains_key(&tweet.author_id) {
                    tweet.author = Some(self.user_cache.get(&tweet.author_id).unwrap().clone());
                } else {
                    debug!("a tweet's author (id={}) is not found", tweet.author_id);
                    continue;
                }

                // get media info
                if let Some(attachments) = &mut tweet.attachments {
                    let media_keys = &attachments.media_keys;
                    attachments.medias = Some(
                        media_keys
                            .iter()
                            .map(|key| media_map.get(key).unwrap().clone())
                            .collect(),
                    );
                };

                let download_info = get_filename_and_url(tweet);
                for info in download_info.iter() {
                    result.push(info.to_owned());
                }
            }
        }
        Ok(result)
    }

    pub async fn get_users_by_ids(&self, user_ids: Vec<&str>) -> Result<Vec<User>> {
        let client = reqwest::Client::new();
        let url: String = "https://api.twitter.com/2/users?ids=".to_string() + &user_ids.join(",");
        let resp = client
            .get(url)
            .headers(self.tweet_auth_header())
            .send()
            .await?;
        if resp.status() == StatusCode::UNAUTHORIZED {
            error!("401: Maybe you have a wrong access_key");
            exit(1);
        }
        let resp = resp.json::<UserResp>().await?;
        Ok(resp.data)
    }

    pub async fn get_users_by_usernames(&self, usernames: Vec<&str>) -> Result<Vec<User>> {
        let client = reqwest::Client::new();
        let url: String =
            "https://api.twitter.com/2/users/by?usernames=".to_string() + &usernames.join(",");
        let resp = client
            .get(url)
            .headers(self.tweet_auth_header())
            .send()
            .await?;
        if resp.status() == StatusCode::UNAUTHORIZED {
            error!("401: Maybe you have a wrong access_key");
            exit(1);
        }
        let resp = resp.json::<UserResp>().await?;
        Ok(resp.data)
    }
}

// -> [(filename, url)]
fn get_filename_and_url(tweet: &Tweet) -> Vec<(String, String)> {
    let mut res: Vec<(String, String)> = vec![];

    let author = tweet.author.as_ref().unwrap();
    let date = &tweet.created_at[0..=9];
    if let Some(attachments) = &tweet.attachments {
        if let Some(medias) = &attachments.medias {
            for (i, media) in medias.iter().enumerate() {
                if let Some(url) = media.url.clone() {
                    let subfix = &regex::Regex::new(r"[a-zA-Z0-9]+$")
                        .unwrap()
                        .captures(&url)
                        .unwrap()[0];
                    let filename = format!(
                        "{}.{}.@{}.{}.{}.{}",
                        date,
                        author.name.replace("/", "[slash]").replace(".", "[dot]"), // '/' is a special char
                        author.username,
                        tweet.id,
                        i,
                        subfix
                    );

                    res.push((filename, url));
                }
            }
        }
    }
    res
}

#[test]
fn regexp_capture_jpg_test() {
    let re = regex::Regex::new(r"[a-zA-Z0-9]+$").unwrap();
    assert!(re.is_match("xx.jpg"));
    assert!(re.is_match("sxx.jpg"));
    let capted = re
        .captures("https://pbs.twimg.com/media/FgFM_DSVsAAfp37.jpg")
        .unwrap();
    assert_eq!(&capted[0], "jpg");
}

#[test]
fn get_filename_and_url_test() {
    let tweet = Tweet {
        text: "#樋口円香生誕祭2022 \n#樋口円香誕生祭2022 https://t.co/yUmWDJwGK1".to_string(),
        created_at: "2022-10-27T05:05:06.000Z".to_string(),
        author_id: "3179724518".to_string(),
        author: Some(User {
            id: "3179724518".to_string(),
            username: "N_ever2_give_up".to_string(),
            name: "ねばえばぎぶあぷ".to_string(),
        }),
        attachments: Some(Attachments {
            media_keys: vec!["3_1585496554525585408".to_string()],
            medias: Some(vec![Media {
                media_key: "3_1585496554525585408".to_string(),
                r#type: "photo".to_string(),
                url: Some("https://pbs.twimg.com/media/FgDQt00acAATk0q.jpg".to_string()),
            }]),
        }),
        id: "1585497795418820609".to_string(),
    };
    let parsed = get_filename_and_url(&tweet);
    assert_eq!(
        parsed[0],
        (
            "2022-10-27.ねばえばぎぶあぷ.@N_ever2_give_up.1585497795418820609.0.jpg".to_string(),
            "https://pbs.twimg.com/media/FgDQt00acAATk0q.jpg".to_string()
        )
    );
}
