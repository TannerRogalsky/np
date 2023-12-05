use futures::{StreamExt, TryFutureExt};
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

    let results = players
        .into_iter()
        .map(|player| {
            let player_id = player.id;
            handle_player(&game, player, client, pool).map_err(move |err| (player_id, err))
        })
        .collect::<futures::stream::FuturesUnordered<_>>()
        .collect::<Vec<Result<_, _>>>()
        .await;

    for result in results {
        if let Err((player_id, err)) = result {
            tracing::error!("FAILED TO FETCH PLAYER {}: {}", player_id, err);
        }
    }

    Ok(())
}

async fn handle_player(
    game: &db::Game,
    player: db::Player,
    client: &reqwest::Client,
    pool: &db::Pool,
) -> Result<(), db::Error> {
    let request = client
        .post("https://np.ironhelmet.com/api")
        .form(&api::APIRequest::v0_1(game.game_id, &player.api_token));
    tracing::trace!("{:?}", request);
    let response = request.send().await.unwrap();
    tracing::trace!("{:?}", response);
    let body = response.text().await.unwrap();
    tracing::debug!("{}", body);
    let parsed = serde_json::from_str::<api::APIResponse>(&body).unwrap();
    tracing::trace!("{:#?}", parsed);

    let tick = db::Tick::fetch_or_insert(parsed.scanning_data.tick, game, pool).await?;

    player
        .save_report(&tick, parsed.scanning_data, pool)
        .await?;

    Ok(())
}
