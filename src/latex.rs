use reqwest;

pub async fn render(code: String) -> Result<Vec<u8>, reqwest::Error> {
    let color = r"{\color{white}";
    let end = "}";
    let data = reqwest::get(
        format!(
            "https://latex.codecogs.com/png.image?{}{}{}",
            color, code, end
        )
        .as_str(),
    )
    .await?
    .bytes()
    .await?;

    Ok(data.to_vec())
}
