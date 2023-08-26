#[tokio::main]
async fn main() {
    if let Err(err) = dotenvy::dotenv() {
        eprintln!("{}", err);
    }

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let api_token = std::env::var("NP_API_TOKEN").unwrap();
    let game_number = std::env::var("NP_GAME_NUMBER").unwrap().parse().unwrap();

    let client = reqwest::Client::new();
    let request = client
        .post("https://np.ironhelmet.com/api")
        .form(&api::APIRequest::v0_1(game_number, api_token));
    tracing::trace!("{:?}", request);
    let response = request.send().await.unwrap();
    tracing::trace!("{:?}", response);
    let body = response.text().await.unwrap();
    tracing::trace!("{}", body);
    let parsed = serde_json::from_str::<api::APIResponse>(&body).unwrap();
    tracing::trace!("{:#?}", parsed);

    std::fs::write(
        format!("{game_number}.json"),
        serde_json::to_string_pretty(&parsed.scanning_data).unwrap(),
    )
    .unwrap();

    // plot(parsed, game_number).unwrap();
}
