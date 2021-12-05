mod app_state;
mod config;
mod kafka;
mod ui;

use app_state::App;

use clap::Parser;

use config::CliOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli_options = CliOptions::parse();
    let mut app = App::new(cli_options);

    let consumer = kafka::consumer_connect(&mut app).await?;
    kafka::topic_partitions(&mut app, &consumer).await?;
    app.partition_counts_from_partitions()?;

    kafka::count_matches(
        consumer,
        app.cli_options().topic.to_owned(),
        app.cli_options().filter.to_owned(),
        app.partition_counts()?,
    )
    .await?;

    ui::run(&app).await?;

    Ok(())
}
