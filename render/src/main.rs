use render::*;

fn main() {
    let root = std::path::Path::new(".");
    let data = serde_json::from_slice::<api::order::Report>(
        &std::fs::read(root.join("out.json")).unwrap(),
    )
    .unwrap();

    let bounds = data.stars.values().fold(AABB::default(), |bounds, star| {
        bounds.extend(star.x, star.y)
    });

    let bounds = match bounds {
        AABB::Unset => panic!("unset bounds"),
        AABB::Set(bounds) => bounds,
    };

    let players = data
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
                fleets: data
                    .fleets
                    .values()
                    .filter(|fleet| fleet.puid == id)
                    .collect(),
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

    let stars = data.stars.values().fold(
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
    document = document.add(stars);

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
                    let target_star = &data.stars[&api::order::ID(order.target_star_uid())];
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
                for order in fleet.o.iter() {
                    let target_star = &data.stars[&api::order::ID(order.target_star_uid())];
                    g = g.add(
                        svg::node::element::Line::new()
                            .set("stroke", player.color)
                            .set("stroke-width", 0.01)
                            .set("x1", fleet.x)
                            .set("y1", fleet.y)
                            .set("x2", target_star.x)
                            .set("y2", target_star.y),
                    );
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

    let star_titles = data.stars.values().fold(
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

    svg::save(root.join("image.svg"), &document).unwrap();
}
