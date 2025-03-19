use reqwest;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Quote {
    pub content: String,
    pub author: String,
}

pub async fn random() -> Result<Quote, reqwest::Error> {
    let quote = reqwest::get("http://api.quotable.io/random")
        .await?
        .json::<Quote>()
        .await?;

    Ok(quote)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_random() {
        let quote = random().await.unwrap();

        assert_ne!(quote.content, "");
        assert_ne!(quote.author, "");
    }
}
