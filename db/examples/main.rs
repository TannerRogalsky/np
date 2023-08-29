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

    let players = game.players(&pool).await.unwrap();
    tracing::info!("{:?}", players);
}
