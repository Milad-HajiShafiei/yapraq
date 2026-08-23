use crate::utils::format_bytes;
use sysinfo::System;

#[derive(Debug)]
pub struct InfoState {
    pub os_name: String,
    pub os_version: String,
    pub kernel: String,
    pub hostname: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub total_ram: String,
}

impl InfoState {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_brand = sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        Self {
            os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            cpu_brand,
            cpu_cores: sys.cpus().len(),
            total_ram: format_bytes(sys.total_memory()),
        }
    }
}

impl Default for InfoState {
    fn default() -> Self {
        Self::new()
    }
}
