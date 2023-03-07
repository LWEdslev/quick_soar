enum SoupError {
    IncorrectURL,
}

pub struct Soup {
    pub html: String,
}

impl Soup {
    pub async fn new(link: String) -> Result<Self, reqwest::Error> {
        let html = reqwest::get(link).await?.text().await?;
        Ok(Self { html })
    }
}