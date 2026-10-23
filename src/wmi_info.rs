use serde::Serialize;
use wmi::WMIConnection;

#[derive(Serialize, Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub driver_version: Option<String>,
    pub driver_date: Option<String>,
    pub resolution: Option<(u32, u32)>,
    pub refresh_rate: Option<u32>,
}

#[derive(Serialize, Debug, Clone)]
pub struct MotherboardInfo {
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub version: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct BiosInfo {
    pub manufacturer: Option<String>,
    pub version: Option<String>,
    pub release_date: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct BatteryInfo {
    pub charge_percent: Option<u8>,
    pub status: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub state: String,
    pub start_mode: String,
}

#[allow(non_snake_case)]
#[derive(serde::Deserialize, Debug)]
struct RawGpu {
    Name: Option<String>,
    DriverVersion: Option<String>,
    DriverDate: Option<String>,
    CurrentHorizontalResolution: Option<u32>,
    CurrentVerticalResolution: Option<u32>,
    CurrentRefreshRate: Option<u32>,
}

#[allow(non_snake_case)]
#[derive(serde::Deserialize, Debug)]
struct RawBaseBoard {
    Manufacturer: Option<String>,
    Product: Option<String>,
    Version: Option<String>,
}

#[allow(non_snake_case)]
#[derive(serde::Deserialize, Debug)]
struct RawBios {
    Manufacturer: Option<String>,
    SMBIOSBIOSVersion: Option<String>,
    ReleaseDate: Option<String>,
}

#[allow(non_snake_case)]
#[derive(serde::Deserialize, Debug)]
struct RawBattery {
    EstimatedChargeRemaining: Option<u8>,
    BatteryStatus: Option<u32>,
}

#[allow(non_snake_case)]
#[derive(serde::Deserialize, Debug)]
struct RawService {
    Name: Option<String>,
    DisplayName: Option<String>,
    State: Option<String>,
    StartMode: Option<String>,
}

fn query_gpus(conn: &WMIConnection) -> Option<Vec<GpuInfo>> {
    let raw: Vec<RawGpu> = conn
        .raw_query("SELECT Name, DriverVersion, DriverDate, CurrentHorizontalResolution, CurrentVerticalResolution, CurrentRefreshRate FROM Win32_VideoController")
        .ok()?;
    Some(raw.into_iter().map(|g| GpuInfo {
        name: g.Name.unwrap_or_else(|| "Unknown".into()),
        driver_version: g.DriverVersion,
        driver_date: g.DriverDate,
        resolution: match (g.CurrentHorizontalResolution, g.CurrentVerticalResolution) {
            (Some(w), Some(h)) => Some((w, h)),
            _ => None,
        },
        refresh_rate: g.CurrentRefreshRate,
    }).collect())
}

fn query_motherboard(conn: &WMIConnection) -> Option<MotherboardInfo> {
    let raw: Vec<RawBaseBoard> = conn.raw_query("SELECT Manufacturer, Product, Version FROM Win32_BaseBoard").ok()?;
    raw.into_iter().next().map(|b| MotherboardInfo {
        manufacturer: b.Manufacturer,
        product: b.Product,
        version: b.Version,
    })
}

fn query_bios(conn: &WMIConnection) -> Option<BiosInfo> {
    let raw: Vec<RawBios> = conn.raw_query("SELECT Manufacturer, SMBIOSBIOSVersion, ReleaseDate FROM Win32_BIOS").ok()?;
    raw.into_iter().next().map(|b| BiosInfo {
        manufacturer: b.Manufacturer,
        version: b.SMBIOSBIOSVersion,
        release_date: b.ReleaseDate,
    })
}

fn query_battery(conn: &WMIConnection) -> Option<BatteryInfo> {
    let raw: Vec<RawBattery> = conn.raw_query("SELECT EstimatedChargeRemaining, BatteryStatus FROM Win32_Battery").ok()?;
    raw.into_iter().next().map(|b| BatteryInfo {
        charge_percent: b.EstimatedChargeRemaining,
        status: b.BatteryStatus.map(|s| match s {
            1 => "Discharging".into(), 2 => "AC Power".into(), 3 => "Fully Charged".into(),
            4 => "Low".into(), 5 => "Critical".into(), 6 => "Charging".into(),
            _ => format!("Unknown({})", s),
        }),
    })
}

fn query_services(conn: &WMIConnection) -> Option<Vec<ServiceInfo>> {
    let raw: Vec<RawService> = conn.raw_query("SELECT Name, DisplayName, State, StartMode FROM Win32_Service").ok()?;
    Some(raw.into_iter().map(|s| ServiceInfo {
        name: s.Name.unwrap_or_default(),
        display_name: s.DisplayName.unwrap_or_default(),
        state: s.State.unwrap_or_else(|| "Unknown".into()),
        start_mode: s.StartMode.unwrap_or_else(|| "Unknown".into()),
    }).collect())
}

pub fn collect_all() -> (Option<Vec<GpuInfo>>, Option<MotherboardInfo>, Option<BiosInfo>, Option<BatteryInfo>, Option<Vec<ServiceInfo>>) {
    let conn = match WMIConnection::new() {
        Ok(c) => c,
        Err(e) => { eprintln!("WMI: connection failed: {e}"); return (None, None, None, None, None); }
    };
    (query_gpus(&conn), query_motherboard(&conn), query_bios(&conn), query_battery(&conn), query_services(&conn))
}