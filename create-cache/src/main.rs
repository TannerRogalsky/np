#[tokio::main]
async fn main() {
    let db = db::init(std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let game = db::Game {
        id: 107,
        game_id: 4725907836895232,
    };

    let most_recent_tick = game.most_recent_tick(&db).await.unwrap();

    let root = std::path::Path::new(".").join("data");
    std::fs::create_dir_all(root.clone()).unwrap();

    for tick in (0..=most_recent_tick).step_by(6) {
        let path = root.join(format!("{}_tick{:0>5}.json", game.game_id, tick));
        if path.exists() {
            continue;
        }

        let reports = game
            .most_recent_universes_for_tick(tick, &db)
            .await
            .unwrap()
            .into_iter()
            .map(|u| u.payload.0)
            .collect::<Vec<_>>();

        std::fs::write(path, serde_json::to_string(&reports).unwrap()).unwrap();
    }
}
