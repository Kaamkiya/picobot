use rand::Rng;
use reqwest;
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Comic {
    /// The year in which the comic was published.
    pub year: String,
    /// The month in which the comic was published (1-indexed).
    pub month: String,
    /// The day of the month when the comic was published (1-indexed).
    pub day: String,

    /// The number ID of the comic.
    pub num: usize,

    pub link: String,
    pub news: String,

    /// The text in the comic, transcribed.
    pub transcript: String,
    /// Alt text for the comic.
    pub alt: String,

    /// The title of the comic.
    pub title: String,
    /// The non-explicit (SFW) title of the comic.
    pub safe_title: String,

    /// A link to the comic's image.
    pub img: String,
}

pub struct Data {
    pub comic: Comic,
    pub imgcontent: Vec<u8>,
}

pub async fn latest() -> Result<Data, reqwest::Error> {
    let comic = reqwest::get("https://xkcd.com/info.0.json")
        .await?
        .json::<Comic>()
        .await?;

    let img = reqwest::get(comic.img.clone()).await?.bytes().await?;

    let data = Data {
        comic,
        imgcontent: img.to_vec(),
    };

    Ok(data)
}

pub async fn random() -> Result<Data, reqwest::Error> {
    let num = reqwest::get("https://xkcd.com/info.0.json")
        .await?
        .json::<Comic>()
        .await?
        .num;

    let comic = reqwest::get(
        format!(
            "https://xkcd.com/{}/info.0.json",
            rand::rng().random_range(1..num)
        )
        .as_str(),
    )
    .await?
    .json::<Comic>()
    .await?;

    let img = reqwest::get(comic.img.clone()).await?.bytes().await?;

    let data = Data {
        comic,
        imgcontent: img.to_vec(),
    };

    Ok(data)
}

pub async fn nth(nth: &str) -> Result<Data, reqwest::Error> {
    let comic = reqwest::get(format!("https://xkcd.com/{}/info.0.json", nth).as_str())
        .await?
        .json::<Comic>()
        .await?;

    let img = reqwest::get(comic.img.clone()).await?.bytes().await?;

    let data = Data {
        comic,
        imgcontent: img.to_vec(),
    };

    Ok(data)
}
