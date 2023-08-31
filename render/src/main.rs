use render::*;

#[tokio::main]
async fn main() {
    let root = std::path::Path::new(".");
    // let full = std::fs::read_to_string(root.join("tick0.json")).unwrap();

    // let reports = full
    //     .lines()
    //     .map(|line| serde_json::from_str::<api::order::Report>(line))
    //     .collect::<Result<Vec<_>, _>>()
    //     .unwrap();

    let db = db::init(std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let game = db::Game {
        id: 107,
        game_id: 4725907836895232,
    };
    // let tick = game.most_recent_tick(&db).await.unwrap();
    let tick = 6;
    let reports = game
        .most_recent_universes_for_tick(tick, &db)
        .await
        .unwrap()
        .into_iter()
        .map(|u| u.payload.0)
        .collect::<Vec<_>>();

    let bounds = reports.iter().fold(AABB::default(), |bounds, report| {
        report
            .stars
            .values()
            .fold(bounds, |bounds, star| bounds.extend(star.x, star.y))
    });

    let bounds = match bounds {
        AABB::Unset => panic!("unset bounds"),
        AABB::Set(bounds) => bounds,
    };

    let mut stars = std::collections::HashMap::<api::order::ID, api::order::Star>::new();
    for report in &reports {
        for (id, reference) in report.stars.iter() {
            match stars.entry(*id) {
                std::collections::hash_map::Entry::Occupied(mut existing) => {
                    if let Some(extra) = reference.extra.as_ref() {
                        existing
                            .get_mut()
                            .extra
                            .get_or_insert_with(|| extra.clone());
                    }
                }
                std::collections::hash_map::Entry::Vacant(v) => {
                    v.insert(reference.clone());
                }
            }
        }
    }

    let fleets = reports
        .iter()
        .flat_map(|report| {
            report
                .fleets
                .iter()
                .map(|(key, value)| (*key, value.clone()))
        })
        .collect::<std::collections::HashMap<_, _>>();

    let players = reports[0]
        .players
        .values()
        .map(|player| {
            let id = match player {
                api::order::Player::Mine(player) => player.uid,
                api::order::Player::Theirs(player) => player.uid,
            };
            Player {
                id,
                color: COLORS[id as usize],
                fleets: fleets.values().filter(|fleet| fleet.puid == id).collect(),
            }
        })
        .collect::<Vec<_>>();

    let mut document = svg::Document::new()
        .set(
            "viewBox",
            (
                bounds.x1 - bounds.width() * 0.1,
                bounds.y1 - bounds.height() * 0.1,
                bounds.width() * 1.2,
                bounds.height() * 1.2,
            ),
        )
        .add(svg::node::element::Style::new(
            [
                ".star { font: 0.05px sans-serif; fill: red; }",
                ".fleet { font: 0.03px sans-serif; fill: white; }",
            ]
            .join("\n"),
        ));

    let star_group = stars.values().fold(
        svg::node::element::Group::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", 0.01),
        |g, star| {
            let path = svg::node::element::Circle::new()
                .set("r", 0.05)
                .set("cx", star.x)
                .set("cy", star.y);

            if star.puid >= 0 {
                g.add(path.set("fill", COLORS[star.puid as usize]))
            } else {
                g.add(path)
            }
        },
    );
    document = document.add(star_group);

    for player in players.iter() {
        let fleets = player.fleets.iter().fold(
            svg::node::element::Group::new().set("fill", player.color),
            |g, fleet| {
                let radius = 0.025;
                // let fleet_node = svg::node::element::Circle::new()
                //     .set("stroke", "black")
                //     .set("stroke-width", 0.01)
                //     .set("r", radius)
                //     .set("cx", fleet.x)
                //     .set("cy", fleet.y);
                let angle = if let Some(order) = fleet.o.first() {
                    let target_star = &stars[&api::order::ID(order.target_star_uid())];
                    (target_star.y - fleet.y).atan2(target_star.x - fleet.x)
                } else {
                    (fleet.y - fleet.ly).atan2(fleet.x - fleet.lx)
                };
                let fleet_node = Triangle {
                    cx: fleet.x,
                    cy: fleet.y,
                    angle,
                    radius,
                }
                .node()
                .set("stroke", "black")
                .set("stroke-width", 0.005);
                let previous_path = svg::node::element::Line::new()
                    .set("stroke", player.color)
                    .set("stroke-width", 0.01)
                    .set("x1", fleet.lx)
                    .set("y1", fleet.ly)
                    .set("x2", fleet.x)
                    .set("y2", fleet.y);
                let mut g = g.add(previous_path);
                let (mut x1, mut y1) = (fleet.x, fleet.y);
                for order in fleet.o.iter() {
                    let target_star = &stars[&api::order::ID(order.target_star_uid())];
                    g = g.add(
                        svg::node::element::Line::new()
                            .set("stroke", player.color)
                            .set("stroke-width", 0.01)
                            .set("x1", x1)
                            .set("y1", y1)
                            .set("x2", target_star.x)
                            .set("y2", target_star.y),
                    );
                    x1 = target_star.x;
                    y1 = target_star.y;
                }
                g.add(fleet_node).add(
                    svg::node::element::Text::new()
                        .set("x", fleet.x)
                        .set("y", fleet.y + radius / 2.)
                        .set("class", "fleet")
                        .set("text-anchor", "middle")
                        .add(svg::node::Text::new(fleet.st.to_string())),
                )
            },
        );
        document = document.add(fleets);
    }

    let star_titles = stars.values().fold(
        svg::node::element::Group::new().set("class", "star"),
        |g, star| {
            let title = svg::node::element::Text::new()
                .set("x", star.x)
                .set("y", star.y - 0.075)
                .set("text-anchor", "middle")
                .add(svg::node::Text::new(&star.n));
            g.add(title)
        },
    );
    document = document.add(star_titles);

    svg::save(
        root.join("svgs")
            .join(format!("{}_tick{:0>5}.svg", game.game_id, tick)),
        &document,
    )
    .unwrap();
}
