use futures::{FutureExt as _, StreamExt, TryFutureExt};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
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

    tracing::debug!("STARTING DB CON");
    let db = db::init(std::env::var("DATABASE_URL")?).await?;

    tracing::debug!("STARTING HANDLER");
    lambda_runtime::run(lambda_runtime::service_fn(|event| {
        handler(event, &client, &db)
    }))
    .await
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Reqwest({0})")]
    Reqwest(#[from] reqwest::Error),
    #[error("Serde({0})")]
    Serde(#[from] serde_json::Error),
    #[error("DB({0})")]
    DB(#[from] db::Error),
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Payload {
    game_id: i64,
}

async fn handler(
    event: lambda_runtime::LambdaEvent<Payload>,
    client: &reqwest::Client,
    pool: &db::Pool,
) -> Result<(), lambda_runtime::Error> {
    tracing::info!("{:?}", event);

    let game_id = event.payload.game_id;
    let game = db::Game::fetch_or_insert(game_id, pool).await?;
    let players = game.players(pool).await?;

    let results = players
        .iter()
        .map(|player| {
            let player_id = player.id;
            fetch_player(&game, player, client).map(move |result| match result {
                Ok(response) => Ok((player, response)),
                Err(err) => Err((player_id, err)),
            })
        })
        .collect::<futures::stream::FuturesUnordered<_>>()
        .collect::<Vec<Result<_, _>>>()
        .await;

    let mut responses = vec![];
    for result in results {
        match result {
            Err((player_id, err)) => {
                tracing::error!("FAILED TO FETCH PLAYER {}: {}", player_id, err);
            }
            Ok(response) => {
                responses.push(response);
            }
        }
    }

    let unique_ticks = responses
        .iter()
        .map(|(_p, r)| r.scanning_data.tick)
        .collect::<std::collections::HashSet<_>>();
    let tick_results = unique_ticks
        .into_iter()
        .map(|tick| db::Tick::fetch_or_insert(tick, &game, pool).map_err(move |err| (tick, err)))
        .collect::<futures::stream::FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await;

    let mut ticks = std::collections::HashMap::new();
    for tr in tick_results {
        match tr {
            Ok(tick) => {
                ticks.insert(tick.tick_id, tick);
            }
            Err((tick, err)) => {
                tracing::error!("FAILED TO INSERT/CREATE TICK {tick}: {err}");
            }
        }
    }

    let results = responses
        .iter()
        .filter_map(|(player, parsed)| {
            if let Some(tick) = ticks.get(&parsed.scanning_data.tick) {
                Some(
                    player
                        .save_report(tick, &parsed.scanning_data, pool)
                        .map_err(move |err| (*player, tick, err)),
                )
            } else {
                tracing::error!(
                    "FAILED TO SAVE UNIVERSE FOR PLAYER({}) AND TICK({}): NO TICK",
                    player.id,
                    parsed.scanning_data.tick
                );
                None
            }
        })
        .collect::<futures::stream::FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await;

    for result in results {
        if let Err((player, tick, err)) = result {
            tracing::error!(
                "FAILED TO SAVE UNIVERSE FOR PLAYER({}) AND TICK({}): {}",
                player.id,
                tick.tick_id,
                err
            );
        }
    }

    Ok(())
}

async fn fetch_player(
    game: &db::Game,
    player: &db::Player,
    client: &reqwest::Client,
) -> Result<api::APIResponse, Error> {
    let request = client
        .post("https://np.ironhelmet.com/api")
        .form(&api::APIRequest::v0_1(game.game_id, &player.api_token));
    tracing::trace!("{:?}", request);
    let response = request.send().await?;
    tracing::trace!("{:?}", response);
    let body = response.text().await?;
    tracing::debug!("{}", body);
    let parsed = serde_json::from_str::<api::APIResponse>(&body)?;
    tracing::trace!("{:#?}", parsed);
    Ok(parsed)
}
