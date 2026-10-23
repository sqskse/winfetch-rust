use serde::Serialize;
use sysinfo::{Disks, Networks, System};

#[derive(Serialize)]
pub struct OsInfo {
    pub hostname: String,
    pub username: String,
    pub name: String,
    pub version: String,
    pub kernel: String,
    pub uptime_secs: u64,
}

#[derive(Serialize)]
pub struct CpuInfo {
    pub brand: String,
    pub cores: usize,
    pub frequency_mhz: u64,
    pub usage_percent: f32,
}

#[derive(Serialize)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub used_mb: u64,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
}

#[derive(Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount: String,
    pub fs: String,
    pub total_gb: f64,
    pub free_gb: f64,
}

#[derive(Serialize)]
pub struct NetworkInfo {
    pub name: String,
    pub mac: String,
    pub ips: Vec<String>,
}

pub fn collect_os() -> OsInfo {
    OsInfo {
        hostname: System::host_name().unwrap_or_else(|| "Unknown".into()),
        // Читаем переменную окружения USERNAME (стандарт для Windows)
        username: std::env::var("USERNAME").unwrap_or_else(|_| "Unknown".into()), // <-- ДОБАВЛЕНО
        name: System::name().unwrap_or_else(|| "Unknown".into()),
        version: System::os_version().unwrap_or_else(|| "Unknown".into()),
        kernel: System::kernel_version().unwrap_or_else(|| "Unknown".into()),
        uptime_secs: System::uptime(),
    }
}

pub fn collect_cpu(sys: &System) -> CpuInfo {
    let cpus = sys.cpus();
    let brand = cpus.first().map(|c| c.brand().to_string()).unwrap_or_else(|| "Unknown".into());
    let freq = cpus.first().map(|c| c.frequency()).unwrap_or(0);
    let usage = if cpus.is_empty() { 0.0 } else { cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32 };
    CpuInfo { brand, cores: cpus.len(), frequency_mhz: freq, usage_percent: usage }
}

pub fn collect_memory(sys: &System) -> MemoryInfo {
    MemoryInfo {
        total_mb: sys.total_memory() / 1024 / 1024,
        used_mb: sys.used_memory() / 1024 / 1024,
        swap_total_mb: sys.total_swap() / 1024 / 1024,
        swap_used_mb: sys.used_swap() / 1024 / 1024,
    }
}

pub fn collect_disks() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    disks.list().iter().map(|d| DiskInfo {
        name: d.name().to_string_lossy().to_string(),
        mount: d.mount_point().to_string_lossy().to_string(),
        fs: d.file_system().to_string_lossy().to_string(),
        total_gb: d.total_space() as f64 / 1024.0 / 1024.0 / 1024.0,
        free_gb: d.available_space() as f64 / 1024.0 / 1024.0 / 1024.0,
    }).collect()
}

pub fn collect_networks() -> Vec<NetworkInfo> {
    let networks = Networks::new_with_refreshed_list();
    networks.list().iter().map(|(name, data)| NetworkInfo {
        name: name.clone(),
        mac: data.mac_address().to_string(),
        ips: data.ip_networks().iter().map(|ip| format!("{}/{}", ip.addr, ip.prefix)).collect(),
    }).collect()
}