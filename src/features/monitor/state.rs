use sysinfo::{Networks, System};

const HISTORY_LENGTH: usize = 100;

#[derive(Debug)]
pub struct MonitorData {
    pub sys: System,
    pub networks: Networks,
    pub cpu_history: Vec<u64>,
    pub memory_history: Vec<u64>,
    pub net_rx_history: Vec<u64>,
    pub net_tx_history: Vec<u64>,
    pub core_usage: Vec<u64>,
    pub mem_total: u64,
    pub mem_used: u64,
    pub mem_usage_percent: f64,
    pub net_rx: u64,
    pub net_tx: u64,
}

impl MonitorData {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let networks = Networks::new_with_refreshed_list();

        Self {
            sys,
            networks,
            cpu_history: vec![0; HISTORY_LENGTH],
            memory_history: vec![0; HISTORY_LENGTH],
            net_rx_history: vec![0; HISTORY_LENGTH],
            net_tx_history: vec![0; HISTORY_LENGTH],
            core_usage: Vec::new(),
            mem_total: 0,
            mem_used: 0,
            mem_usage_percent: 0.0,
            net_rx: 0,
            net_tx: 0,
        }
    }

    pub fn update(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.networks.refresh(true);

        let global_cpu = self.sys.global_cpu_usage().round() as u64;
        push_history(&mut self.cpu_history, global_cpu);
        self.core_usage = self
            .sys
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage().round() as u64)
            .collect();

        self.mem_total = self.sys.total_memory();
        self.mem_used = self.sys.used_memory();
        if self.mem_total > 0 {
            self.mem_usage_percent = (self.mem_used as f64 / self.mem_total as f64) * 100.0;
        }
        push_history(
            &mut self.memory_history,
            self.mem_usage_percent.round() as u64,
        );

        let mut rx = 0;
        let mut tx = 0;
        for network_data in self.networks.values() {
            rx += network_data.total_received();
            tx += network_data.total_transmitted();
        }
        push_history(
            &mut self.net_rx_history,
            rx.saturating_sub(self.net_rx) / 1024,
        );
        push_history(
            &mut self.net_tx_history,
            tx.saturating_sub(self.net_tx) / 1024,
        );
        self.net_rx = rx;
        self.net_tx = tx;
    }
}

fn push_history(history: &mut Vec<u64>, value: u64) {
    if history.len() >= HISTORY_LENGTH {
        history.remove(0);
    }
    history.push(value);
}

impl Default for MonitorData {
    fn default() -> Self {
        Self::new()
    }
}
