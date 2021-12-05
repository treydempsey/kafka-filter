use crate::app_state::{App, PartitionCounts};

use futures::future::{err, ok};
use futures::TryStreamExt;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::Message;

use std::time::Duration;

pub async fn consumer_connect(app: &mut App) -> anyhow::Result<StreamConsumer> {
    let cli_options = &app.cli_options();
    let consumer = ClientConfig::new()
        .set("group.id", &cli_options.group_id)
        .set("bootstrap.servers", &cli_options.bootstrap_servers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", &cli_options.offset.to_string())
        .create()?;

    Ok(consumer)
}

pub async fn topic_partitions(app: &mut App, consumer: &StreamConsumer) -> anyhow::Result<()> {
    let cli_options = &app.cli_options();
    let timeout = Duration::from_secs(5);
    let metadata = &consumer.fetch_metadata(Some(&cli_options.topic), timeout)?;
    let topics = metadata.topics();

    if topics.len() == 1 {
        let topic = topics.get(0).ok_or_else(|| {
            anyhow::anyhow!("Expected topics.get(0) to return a topic given len() == 1")
        })?;
        let partitions = topic
            .partitions()
            .iter()
            .map(|p| p.id())
            .collect::<Vec<i32>>();

        app.set_partitions(partitions);

        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Error fetching metadata for topic: {}. Found {} topics.",
            &cli_options.topic,
            topics.len()
        ))
    }
}

pub async fn count_matches(
    consumer: StreamConsumer,
    topic: String,
    filter: String,
    partition_counts: PartitionCounts,
) -> anyhow::Result<()> {
    // Main consume loop
    tokio::spawn(async move {
        consumer
            .subscribe(&[&topic])
            .expect("Error subscribing to topic");
        consumer
            .stream()
            .try_for_each(|message| {
                let mut needle =
                    jq_rs::compile(&filter).expect("Error compiling jq search pattern");
                if let Some(Ok(view)) = message.payload_view::<str>() {
                    if let Ok(m) = &needle.run(view) {
                        if m != "null\n" {
                            // "null\n" is returned for no match
                            let partition = message.partition().to_string();
                            if let Some(mut entry) = partition_counts.get_mut(&partition) {
                                // Update existing partition entry
                                *entry += 1;
                            } else {
                                // New partition entry
                                partition_counts.insert(partition, 1);
                            }
                        }
                    }
                }

                match consumer.commit_message(&message, CommitMode::Async) {
                    Ok(_) => ok(()),
                    Err(e) => err(e),
                }
            })
            .await
            .expect("Error consuming from topic");
    });

    Ok(())
}
