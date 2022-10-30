pub struct UrlBuilder {
    host: String,
    params: Vec<(String, String)>,
}

impl UrlBuilder {
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_string(),
            params: vec![],
        }
    }

    pub fn param(&mut self, k: &str, v: &str) -> &mut Self {
        self.params.push((k.to_string(), v.to_string()));
        self
    }

    pub fn get_url(&self) -> String {
        let param_str = self
            .params
            .iter()
            .map(|param| format!("{}={}", param.0, param.1))
            .collect::<Vec<String>>()
            .join("&");
        self.host.clone() + "?" + param_str.as_str()
    }
}

#[test]
fn url_test() {
    let url = UrlBuilder::new("https://localhost/api")
        .param("k", "v")
        .param("k2.1", "1,2,3")
        .get_url();
    assert_eq!("https://localhost/api?k=v&k2.1=1,2,3", url);

    let url2 = UrlBuilder::new("https://api.twitter.com/2/users/114514/liked_tweets")
        .param("expansions", "attachments.media_keys")
        .param("media.fields", "url")
        .param("tweet.fields", "created_at,author_id")
        .param("max_results", "60")
        .get_url();
    assert_eq!("https://api.twitter.com/2/users/114514/liked_tweets?expansions=attachments.media_keys&media.fields=url&tweet.fields=created_at,author_id&max_results=60", url2);
}
