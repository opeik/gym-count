use std::collections::HashMap;

use actix_web::{error, get, App, HttpResponse, HttpServer};
use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::format::FmtSpan;

static GYM_MEMBER_COUNT_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(r#"span[data-live-count]"#).unwrap());
const GYM_COUNT_URL: &str = "https://revofitness.com.au/livemembercount/";

type GymState = HashMap<String, u32>;

#[actix_web::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .init();

    HttpServer::new(|| {
        App::new()
            .service(gym_member_count)
            .wrap(TracingLogger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}

#[get("/gym_member_count")]
async fn gym_member_count() -> actix_web::Result<HttpResponse> {
    let gym_state = get_gym_member_count()
        .await
        .map_err(|e| error::ErrorInternalServerError(e.to_string()))?;
    Ok(HttpResponse::Ok().json(gym_state))
}

async fn get_gym_member_count() -> Result<GymState> {
    info!("fetching gym state...");
    let response = reqwest::get(GYM_COUNT_URL)
        .await
        .context("failed to request gym state")?;
    let body = response
        .text()
        .await
        .context("failed to receive gym state response")?;
    let html = Html::parse_document(&body);
    let gym_state = extract_gym_member_count(&html)?;
    info!("successfully fetched gym member count!");

    Ok(gym_state)
}

fn extract_gym_member_count(html: &Html) -> Result<GymState> {
    html.select(&GYM_MEMBER_COUNT_SELECTOR)
        .map(|gym_element| {
            let name = gym_element
                .attr("data-live-count")
                .context("failed to get gym name")?;
            let member_count = gym_element
                .text()
                .next()
                .context("failed to get gym member count")?
                .parse::<u32>()
                .context("failed to parse gym member count")?;
            Ok((name.to_string(), member_count))
        })
        .collect::<Result<GymState>>()
}
