use std::sync::LazyLock;

use anyhow::Ok;
use dashmap::DashMap;
use wreq::Client;
use wreq_util::{Emulation, EmulationOS, EmulationOption};

pub static CLIENT_POOL: LazyLock<ClientPool> = LazyLock::new(|| ClientPool::new());

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ClientProfile {
    pub browser: Emulation,
    pub os: EmulationOS,
}

impl ClientProfile {
    pub fn new(browser: Emulation, os: EmulationOS) -> Self {
        Self { browser, os }
    }
}

pub struct ClientPool {
    clients: DashMap<ClientProfile, Client>,
}

impl ClientPool {
    pub fn new() -> Self {
        Self {
            clients: DashMap::new(),
        }
    }

    pub fn get(&self, profile: &ClientProfile) -> anyhow::Result<Client> {
        if let Some(client) = self.clients.get(profile) {
            return Ok(client.clone());
        }

        println!("building profile {:?}", profile);

        let client = Client::builder()
            .emulation(
                EmulationOption::builder()
                    .emulation(profile.browser)
                    .emulation_os(profile.os)
                    .build(),
            )
            .build()?;

        self.clients.insert(profile.clone(), client.clone());

        Ok(client)
    }
}
