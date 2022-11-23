use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Attachments {
    pub media_keys: Vec<String>,
    pub medias: Option<Vec<Media>>, // for data join
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tweet {
    pub text: String,
    pub created_at: String,
    pub author_id: String,
    pub author: Option<User>,
    pub attachments: Option<Attachments>,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Media {
    pub media_key: String,
    pub r#type: String,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Includes {
    pub media: Vec<Media>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TweetResp {
    pub data: Option<Vec<Tweet>>,
    pub includes: Option<Includes>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResp {
    pub data: Option<Vec<User>>,
    pub errors: Option<Vec<UserErr>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserErr {
    pub value: String,
    pub detail: String,
    pub title: String,
    pub resource_type: String,
    pub parameter: String,
    pub resource_id: String,
    pub r#type: String,
}
