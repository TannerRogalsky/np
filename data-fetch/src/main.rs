use futures::TryStreamExt;
use lambda_runtime::{Error, LambdaEvent};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    if let Err(err) = dotenvy::dotenv() {
        eprintln!("{}", err);
    }

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "data-fetch=debug".into()),
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .without_time()
                .with_ansi(false),
        )
        .init();

    let client = reqwest::Client::new();

    let db = db::init(std::env::var("DATABASE_URL")?).await?;

    lambda_runtime::run(lambda_runtime::service_fn(|event| {
        handler(event, &client, &db)
    }))
    .await
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Payload {
    game_id: i64,
}

async fn handler(
    event: LambdaEvent<Payload>,
    client: &reqwest::Client,
    pool: &db::Pool,
) -> Result<(), Error> {
    tracing::info!("{:?}", event);

    let game_id = event.payload.game_id;
    let game = db::Game::fetch_or_insert(game_id, pool).await?;
    let players = game.players(pool).await?;

    players
        .into_iter()
        .map(|player| handle_player(game_id, player, client, pool))
        .collect::<futures::stream::FuturesUnordered<_>>()
        .try_collect::<Vec<_>>()
        .await?;

    Ok(())
}

async fn handle_player(
    game_id: i64,
    player: db::Player,
    client: &reqwest::Client,
    pool: &db::Pool,
) -> Result<(), lambda_runtime::Error> {
    let request = client
        .post("https://np.ironhelmet.com/api")
        .form(&api::APIRequest::v0_1(game_id, &player.api_token));
    tracing::trace!("{:?}", request);
    let response = request.send().await.unwrap();
    tracing::trace!("{:?}", response);
    let body = response.text().await.unwrap();
    tracing::debug!("{}", body);
    let parsed = serde_json::from_str::<api::APIResponse>(&body).unwrap();
    tracing::trace!("{:#?}", parsed);

    player.save_report(parsed.scanning_data, pool).await?;

    Ok(())
}
