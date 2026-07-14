use std::sync::Arc;
use chrono::Utc;
use control_core::RuntimeEventKind;
use crate::SharedState;

pub async fn run(state: SharedState) {
    let mut rx = state.data_tx.subscribe();

    // create local copy we can modify and then clone
    let mut machines = (*state.machines.load_full()).clone();

    loop {
        println!("sync_machine_registry started");
        let Ok(export) = rx.recv().await else {
            println!("sync_machine_registry exiting");
            // tx died, meaning we should exit
            return;
        };

        let mut modified: bool = false;

        for event in &export.runtime_events {
            match event.kind {
                RuntimeEventKind::MachineConnected(ident) => {
                    let Some((last_active, connected)) = machines.get_mut(&ident) else {
                        println!("sync_machine_registry: registering connected machine");

                        // machine not registered yet.
                        machines.insert(ident, (Utc::now(), true));
                        modified = true;
                        continue;
                    };

                    // already marked as connected
                    if *connected { continue; }

                    // set
                    *connected = true;
                    *last_active = Utc::now();
                    modified = true;
                },
                RuntimeEventKind::MachineDisconnected(ident) => {
                    let Some((_, connected)) = machines.get_mut(&ident) else {
                        println!("sync_machine_registry: registering disconnected machine");
                        // machine not registered yet.
                        machines.insert(ident, (Utc::now(), true));
                        modified = true;
                        continue;
                    };

                    // already marked as connected
                    if !*connected { continue; }

                    // set
                    *connected = false;
                    modified = true;
                },
            }
        }

        if modified {
            println!("Machine Registry changed!");

            // put new registry so everyone can see it now
            state.machines.swap(Arc::new(machines.clone()));

            // write the changes into the database
            // TODO:
        }
    }
}
