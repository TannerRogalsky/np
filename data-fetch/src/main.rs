use api::order::{OrderRequest, OrderResponse};
use base64::Engine;
use plotters::prelude::*;

// const GAME_NUMBER: u64 = 5318173459218432;
const GAME_NUMBER: u64 = 6258592149405696;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let user_info = r#"GET AND DECODE FROM COOKIE"#;
    let encoded = base64::engine::general_purpose::STANDARD.encode(user_info);
    // for some reason the non-hex base64 characters are escaped to octal
    let encoded = std::borrow::Cow::Owned(encoded.replace("=", "\\075"));
    let some_value_1 = "SECOND PART OF COOKIE".into();
    let some_value_2 = "THIRD PART OF COOKIE".into();

    let cookie = format!(
        "auth=\"\"{}\"\"",
        [encoded, some_value_1, some_value_2].join("|")
    );
    let url = "https://np.ironhelmet.com".parse::<reqwest::Url>().unwrap();

    let jar = reqwest::cookie::Jar::default();
    jar.add_cookie_str(&cookie, &url);

    let client = reqwest::Client::builder()
        .cookie_provider(std::sync::Arc::new(jar))
        .build()
        .unwrap();

    let request = client
        .post("https://np.ironhelmet.com/trequest_game/order")
        .form(&OrderRequest::full_universe_report(GAME_NUMBER));
    tracing::trace!("{:?}", request);
    let response = request.send().await.unwrap();
    tracing::trace!("{:?}", response);
    let body = response.text().await.unwrap();
    tracing::trace!("{}", body);
    let parsed = serde_json::from_str::<OrderResponse>(&body).unwrap();
    tracing::trace!("{:#?}", parsed);

    std::fs::write(
        format!("{GAME_NUMBER}.json"),
        serde_json::to_string_pretty(&parsed.report).unwrap(),
    )
    .unwrap();

    plot(parsed).unwrap();
}

fn plot(data: OrderResponse) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("{GAME_NUMBER}.png");
    let root = BitMapBackend::new(&path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let stars = data
        .report
        .stars
        .values()
        .map(|star| (star.x, star.y))
        .collect::<Vec<_>>();

    let (start_x, start_y) = stars[0];
    let (x_spec, y_spec) = stars.iter().copied().fold(
        (start_x..start_x, start_y..start_y),
        |(x_spec, y_spec), (x, y)| {
            (
                x_spec.start.min(x)..(x_spec.end.max(x)),
                y_spec.start.min(y)..(y_spec.end.max(y)),
            )
        },
    );
    let y_spec = y_spec.end..y_spec.start;

    struct Player {
        color: RGBColor,
        fleets: Vec<api::order::Fleet>,
    }

    let colors = vec![RED, BLUE, YELLOW, CYAN, MAGENTA];
    let players = colors
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(index, color)| {
            if let Some(_player) = data.report.players.get(&api::order::ID(index as _)) {
                Some(Player {
                    color,
                    fleets: data
                        .report
                        .fleets
                        .values()
                        .filter(|fleet| fleet.puid == index as i32)
                        .cloned()
                        .collect(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let areas = root.split_by_breakpoints([944], [80]);

    let mut x_hist_ctx = ChartBuilder::on(&areas[0])
        .y_label_area_size(40)
        .build_cartesian_2d(
            x_spec.clone().step(0.01).use_round().into_segmented(),
            0..250,
        )?;
    let mut y_hist_ctx = ChartBuilder::on(&areas[3])
        .x_label_area_size(40)
        .build_cartesian_2d(0..250, y_spec.clone().step(-0.01).use_round())?;
    let mut scatter_ctx = ChartBuilder::on(&areas[2])
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(x_spec, y_spec)?;
    scatter_ctx
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;
    scatter_ctx.draw_series(data.report.stars.values().map(|star| {
        let color = colors.get(star.puid as usize).copied().unwrap_or(GREEN);
        Circle::new((star.x, star.y), 2, color.filled())
    }))?;

    for player in players {
        for fleet in player.fleets.iter() {
            scatter_ctx.draw_series(LineSeries::new(
                [(fleet.x, fleet.y), (fleet.lx, fleet.ly)],
                player.color,
            ))?;

            for order in fleet.o.iter() {
                let star = data
                    .report
                    .stars
                    .get(&api::order::ID(order.target_star_uid()))
                    .unwrap();
                scatter_ctx.draw_series(LineSeries::new(
                    [(fleet.x, fleet.y), (star.x, star.y)],
                    player.color,
                ))?;
            }
        }
        scatter_ctx.draw_series(player.fleets.iter().map(|fleet| {
            TriangleMarker::new((fleet.x, fleet.y), fleet.st, player.color.filled())
        }))?;
    }
    let x_hist = Histogram::vertical(&x_hist_ctx)
        .style(GREEN.filled())
        .margin(0)
        .data(stars.iter().map(|(x, _)| (*x, 1)));
    let y_hist = Histogram::horizontal(&y_hist_ctx)
        .style(GREEN.filled())
        .margin(0)
        .data(stars.iter().map(|(_, y)| (*y, 1)));
    x_hist_ctx.draw_series(x_hist)?;
    y_hist_ctx.draw_series(y_hist)?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present()?;
    tracing::info!("Result has been saved");

    Ok(())
}
