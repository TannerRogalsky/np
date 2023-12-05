#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let db_url = std::env::var("DATABASE_URL").unwrap();
    let pool = db::init(db_url).await.unwrap();

    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(row.0, 150);

    let game = db::Game::fetch_or_insert(1234, &pool).await.unwrap();
    tracing::info!("{:?}", game);

    let tick = db::Tick::fetch_or_insert(0, &game, &pool).await.unwrap();
    tracing::info!("{:?}", tick);

    let players = game.players(&pool).await.unwrap();
    for player in players.iter() {
        tracing::info!("{:?}", player);
        player
            .save_report(
                &tick,
                &TestReport {
                    foo: 123,
                    bar: vec![1, 2],
                },
                &pool,
            )
            .await
            .unwrap();
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TestReport {
    foo: u32,
    bar: Vec<u32>,
}
