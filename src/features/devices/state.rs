use rusb::{Context, UsbContext};

#[derive(Debug, Clone)]
pub struct UsbDeviceInfo {
    pub bus: u8,
    pub address: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: String,
    pub product: String,
}

#[derive(Debug)]
pub struct DevicesState {
    pub devices: Vec<UsbDeviceInfo>,
    pub selected: usize,
}

impl DevicesState {
    pub fn new() -> Self {
        let mut state = Self {
            devices: Vec::new(),
            selected: 0,
        };
        state.refresh();
        state
    }

    pub fn select_next(&mut self) {
        if !self.devices.is_empty() {
            self.selected = (self.selected + 1).min(self.devices.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn refresh(&mut self) {
        let mut devices = Vec::new();

        let context = match Context::new() {
            Ok(ctx) => ctx,
            Err(_) => {
                self.devices.clear();
                self.selected = 0;
                return;
            }
        };

        if let Ok(device_list) = context.devices() {
            for device in device_list.iter() {
                let desc = match device.device_descriptor() {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                let mut manufacturer = "Unknown".to_string();
                let mut product = "Unknown".to_string();

                if let Ok(handle) = device.open() {
                    let timeout = std::time::Duration::from_secs(1);
                    if let Ok(languages) = handle.read_languages(timeout)
                        && let Some(&lang) = languages.first()
                    {
                        manufacturer = handle
                            .read_manufacturer_string(lang, &desc, timeout)
                            .unwrap_or(manufacturer);
                        product = handle
                            .read_product_string(lang, &desc, timeout)
                            .unwrap_or(product);
                    }
                }

                devices.push(UsbDeviceInfo {
                    bus: device.bus_number(),
                    address: device.address(),
                    vendor_id: desc.vendor_id(),
                    product_id: desc.product_id(),
                    manufacturer,
                    product,
                });
            }
        }

        devices.sort_by_key(|device| (device.bus, device.address));
        self.devices = devices;
        self.selected = self.selected.min(self.devices.len().saturating_sub(1));
    }
}

impl Default for DevicesState {
    fn default() -> Self {
        Self::new()
    }
}
