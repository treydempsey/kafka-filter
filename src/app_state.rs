use crate::config::CliOptions;

use dashmap::DashMap;

use std::sync::Arc;

pub type PartitionCounts = Arc<DashMap<String, u64>>;

pub trait PartitionCountsExt {
    fn from_partitions(_: &[i32]) -> Self;
}

impl PartitionCountsExt for PartitionCounts {
    fn from_partitions(partitions: &[i32]) -> Self {
        let partition_counts = Arc::new(DashMap::with_capacity(partitions.len()));
        for partition in partitions {
            let partition = (*partition).to_string();
            partition_counts.insert(partition, 0);
        }

        partition_counts
    }
}

pub struct App {
    cli_options: CliOptions,
    partitions: Option<Vec<i32>>,
    partition_counts: Option<PartitionCounts>,
}

impl App {
    pub fn new(cli_options: CliOptions) -> Self {
        Self {
            cli_options,
            partitions: None,
            partition_counts: None,
        }
    }

    pub fn cli_options(&self) -> &CliOptions {
        &self.cli_options
    }

    pub fn set_partitions(&mut self, partitions: Vec<i32>) {
        self.partitions = Some(partitions);
    }

    pub fn partitions(&self) -> anyhow::Result<&Vec<i32>> {
        self.partitions.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "Error accessing partitions from App state. It is not set at this point."
            )
        })
    }

    pub fn partition_counts_from_partitions(&mut self) -> anyhow::Result<()> {
        self.partition_counts = Some(PartitionCounts::from_partitions(self.partitions()?));
        Ok(())
    }

    pub fn partition_counts(&self) -> anyhow::Result<PartitionCounts> {
        Ok(self
            .partition_counts
            .as_ref()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Error accessing partition_counts from App state. It is not set at this point."
                )
            })?
            .clone())
    }
}
