use std::{sync::Arc, time::{Duration, Instant}};
use tokio::{sync::broadcast, time::timeout};
use clickhouse::{Client, insert::Insert};
use control_core::RuntimeExport;
use crate::SharedState;

mod types;
use types::ScalarValueColumns;
use types::ConfigMutationRecordRow;

pub(crate) async fn run(state: SharedState) -> anyhow::Result<()> {
    let export_interval = state.config.export_interval;

    let mut rx = state.data_tx.subscribe();
    let mut last_export_ts = Instant::now();

    loop {
        let mut inserts = Inserts::new(&state.client).await?;

        loop {
            let now = Instant::now();

            if now.duration_since(last_export_ts) >= export_interval {
                println!("Exporting");
                inserts.end().await?;
                last_export_ts = now;
                break;
            }

            if let Ok(result) = timeout(Duration::from_millis(100), rx.recv()).await {
                use broadcast::error::RecvError;

                match result {
                    Ok(data) => map_export(&mut inserts, data).await?,
                    Err(e) => match e {
                        RecvError::Closed => return Ok(()),
                        RecvError::Lagged(count) => {
                            eprintln!("Lagged behind {count} messages!");
                            continue;
                        },
                    },
                }
            }
        }
    }
}

struct Inserts {
    pub config_mutations: Insert<ConfigMutationRecordRow>,
}

impl Inserts {
    pub async fn new(client: &Client) -> clickhouse::error::Result<Self> {
        let config_mutations = client.insert::<ConfigMutationRecordRow>("config_mutations")?;

        Ok(Self { config_mutations })
    }

    pub async fn end(self) -> clickhouse::error::Result<()> {
        tokio::try_join!(
            self.config_mutations.end(),
        )?;

        Ok(())
    }
}

async fn map_export(
    inserts: &mut Inserts,
    data: Arc<RuntimeExport>,
) -> clickhouse::error::Result<()> {
    for item in &data.config_mutations {
        let ScalarValueColumns { 
            value_type, 
            value_string, 
            value_int, 
            value_float, 
            value_bool 
        } = ScalarValueColumns::from(&item.value);
        
        inserts.config_mutations.write(&ConfigMutationRecordRow {
            timestamp: item.timestamp,
            ident_vendor: item.ident.vendor,
            ident_machine: item.ident.machine,
            ident_serial: item.ident.serial,
            name: item.name.to_string(),
            value_type,
            value_string,
            value_int,
            value_float,
            value_bool,
            origin: item.origin,
            result: item.result,
        }).await?;
    }

    Ok(())
}
