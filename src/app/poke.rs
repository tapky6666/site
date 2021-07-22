use color_eyre::eyre::Result;
use std::{env, time::Duration};
use tokio::time::sleep as delay_for;

#[instrument(err)]
pub async fn the_cloud() -> Result<()> {
    info!("waiting for things to settle");
    delay_for(Duration::from_secs(10)).await;

    info!("purging cloudflare cache");
    cloudflare().await?;

    info!("waiting for the cloudflare cache to purge globally");
    delay_for(Duration::from_secs(45)).await;

    info!("poking mi");
    mi().await?;

    info!("poking bing");
    bing().await?;

    info!("poking google");
    google().await?;

    Ok(())
}

#[instrument(err)]
async fn bing() -> Result<()> {
    let cli = reqwest::Client::new();
    cli.get("https://www.bing.com/ping")
        .query(&[("sitemap", "https://fetsorn.website/sitemap.xml")])
        .header("User-Agent", crate::APPLICATION_NAME)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[instrument(err)]
async fn google() -> Result<()> {
    let cli = reqwest::Client::new();
    cli.get("https://www.google.com/ping")
        .query(&[("sitemap", "https://fetsorn.website/sitemap.xml")])
        .header("User-Agent", crate::APPLICATION_NAME)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[instrument(err)]
async fn cloudflare() -> Result<()> {
    let cli = cfcache::Client::new(env::var("CF_TOKEN")?, env::var("CF_ZONE_ID")?)?;
    cli.purge(
        vec![
            "https://fetsorn.website/sitemap.xml",
            "https://fetsorn.website",
            "https://fetsorn.website/blog",
            "https://fetsorn.website/blog.atom",
            "https://fetsorn.website/blog.json",
            "https://fetsorn.website/blog.rss",
            "https://fetsorn.website/gallery",
            "https://fetsorn.website/talks",
            "https://fetsorn.website/resume",
            "https://fetsorn.website/signalboost",
            "https://fetsorn.website/feeds",
        ]
        .into_iter()
        .map(|i| i.to_string())
        .collect(),
    )
    .await?;

    Ok(())
}

#[instrument(err)]
async fn mi() -> Result<()> {
    let cli = mi::Client::new(env::var("MI_TOKEN")?, crate::APPLICATION_NAME.to_string())?;
    cli.refresh().await?;

    Ok(())
}
