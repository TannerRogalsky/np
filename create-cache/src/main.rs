#[tokio::main]
async fn main() {
    let ticks = vec![
        0, 6, 12, 18, 24, 30, 36, 42, 48, 54, 60, 66, 72, 78, 84, 90, 96, 102, 108, 114, 120, 126,
        132, 138, 144, 150, 156, 162, 168, 174, 180, 186, 192, 198, 204, 210, 216, 222, 228, 234,
        240, 246, 252, 258, 264, 270, 276, 282, 288, 294, 300, 306, 312, 318, 324, 330, 336, 342,
        348, 354, 360, 366, 372, 378, 384, 390, 396, 402, 408, 414, 420, 426, 432, 438, 444, 450,
        456, 462, 468, 474, 480, 486, 492, 498, 504, 510, 516, 522, 528, 534, 540, 546, 552, 558,
        564, 570, 576, 582,
    ];

    let db = db::init(std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let game = db::Game {
        id: 107,
        game_id: 4725907836895232,
    };

    let root = std::path::Path::new(".").join("data");
    std::fs::create_dir_all(root.clone()).unwrap();

    for tick in ticks {
        let reports = game
            .most_recent_universes_for_tick(tick, &db)
            .await
            .unwrap()
            .into_iter()
            .map(|u| u.payload.0)
            .collect::<Vec<_>>();
        let path = root.join(format!("{}_tick{:0>5}.json", game.game_id, tick));
        std::fs::write(path, serde_json::to_string(&reports).unwrap()).unwrap();
    }
}
