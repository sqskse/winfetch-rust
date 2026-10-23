mod sys_info;
mod wmi_info;
mod software_info;

use anyhow::Result;
use clap::Parser;
use serde::Serialize;
use std::fmt::Write;
use std::process::ExitCode;
use sysinfo::System;

#[derive(Parser)]
#[command(name = "winfetch", version, about = "System inventory for Windows admins")]
struct Cli {
    #[arg(long)]
    json: bool,
    
    #[arg(long, value_delimiter = ',')]
    section: Option<Vec<String>>,
    
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Serialize)]
struct Report {
    os: sys_info::OsInfo,
    cpu: sys_info::CpuInfo,
    memory: sys_info::MemoryInfo,
    disks: Vec<sys_info::DiskInfo>,
    network: Vec<sys_info::NetworkInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gpus: Option<Vec<wmi_info::GpuInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    motherboard: Option<wmi_info::MotherboardInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bios: Option<wmi_info::BiosInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    battery: Option<wmi_info::BatteryInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    services: Option<Vec<wmi_info::ServiceInfo>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    software: Vec<software_info::SoftwareInfo>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => { eprintln!("error: {e:#}"); ExitCode::FAILURE }
    }
}

fn run(cli: Cli) -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os = sys_info::collect_os();
    let cpu = sys_info::collect_cpu(&sys);
    let memory = sys_info::collect_memory(&sys);
    let disks = sys_info::collect_disks();
    let network = sys_info::collect_networks();

    let (gpus, motherboard, bios, battery, services) = wmi_info::collect_all();
    let software = software_info::collect_installed_software();

    let report = Report { os, cpu, memory, disks, network, gpus, motherboard, bios, battery, services, software };

    let is_json = cli.json || cli.output.as_ref().map_or(false, |p| p.ends_with(".json"));
    
    let content = if is_json {
        serde_json::to_string_pretty(&report)?
    } else {
        format_human(&report, cli.section.as_deref())
    };

    if let Some(path) = cli.output {
        std::fs::write(&path, content)?;
        println!("Отчет успешно сохранен в файл: {}", path);
    } else {
        print!("{}", content);
    }

    Ok(())
}

fn format_human(r: &Report, section: Option<&[String]>) -> String {
    let mut out = String::new();
    let want = |name: &str| section.map_or(true, |s| s.iter().any(|x| x == name));
    let sep = |out: &mut String| { let _ = writeln!(out, "{}", "─".repeat(55)); };

        if want("os") {
        sep(&mut out); 
        let _ = writeln!(out, "OS");
        let _ = writeln!(out, "  Hostname : {}", r.os.hostname);
        let _ = writeln!(out, "  User     : {}", r.os.username); // <-- ДОБАВЛЕНО
        let _ = writeln!(out, "  Name     : {}", r.os.name);
        let _ = writeln!(out, "  Version  : {}", r.os.version);
        let _ = writeln!(out, "  Kernel   : {}", r.os.kernel);
        let _ = writeln!(out, "  Uptime   : {}", fmt_uptime(r.os.uptime_secs));
    }

    if want("cpu") {
        sep(&mut out); 
        let _ = writeln!(out, "CPU");
        let _ = writeln!(out, "  Brand    : {}", r.cpu.brand);
        let _ = writeln!(out, "  Cores    : {}", r.cpu.cores);
        let _ = writeln!(out, "  Freq     : {} MHz", r.cpu.frequency_mhz);
        let _ = writeln!(out, "  Usage    : {:.1}%", r.cpu.usage_percent);
    }

    if want("memory") {
        sep(&mut out); 
        let _ = writeln!(out, "MEMORY");
        let _ = writeln!(out, "  RAM      : {} MB used / {} MB total", r.memory.used_mb, r.memory.total_mb);
        let _ = writeln!(out, "  Swap     : {} MB used / {} MB total", r.memory.swap_used_mb, r.memory.swap_total_mb);
    }

    if want("disk") {
        sep(&mut out); 
        let _ = writeln!(out, "DISKS");
        for d in &r.disks {
            let _ = writeln!(out, "  [{}] {} ({}) — {:.1} GB free / {:.1} GB total", d.mount, d.name, d.fs, d.free_gb, d.total_gb);
        }
    }

    if want("network") {
        sep(&mut out); 
        let _ = writeln!(out, "NETWORK");
        for n in &r.network {
            let _ = writeln!(out, "  {} (MAC: {})", n.name, n.mac);
            for ip in &n.ips { let _ = writeln!(out, "      {}", ip); }
        }
    }

    if want("gpu") {
        if let Some(gpus) = &r.gpus {
            sep(&mut out); 
            let _ = writeln!(out, "GPU");
            for g in gpus {
                let _ = writeln!(out, "  {}", g.name);
                if let Some(v) = &g.driver_version { let _ = writeln!(out, "      Driver   : {}", v); }
                if let Some(v) = &g.driver_date { let _ = writeln!(out, "      Date     : {}", v); }
                if let Some((w, h)) = g.resolution { let _ = writeln!(out, "      Res      : {}x{}", w, h); }
                if let Some(rr) = g.refresh_rate { let _ = writeln!(out, "      Refresh  : {} Hz", rr); }
            }
        }
    }

    if want("motherboard") {
        if let Some(mb) = &r.motherboard {
            sep(&mut out); 
            let _ = writeln!(out, "MOTHERBOARD");
            if let Some(v) = &mb.manufacturer { let _ = writeln!(out, "  Manufacturer : {}", v); }
            if let Some(v) = &mb.product      { let _ = writeln!(out, "  Product      : {}", v); }
            if let Some(v) = &mb.version      { let _ = writeln!(out, "  Version      : {}", v); }
        }
    }

    if want("bios") {
        if let Some(b) = &r.bios {
            sep(&mut out); 
            let _ = writeln!(out, "BIOS");
            if let Some(v) = &b.manufacturer { let _ = writeln!(out, "  Manufacturer : {}", v); }
            if let Some(v) = &b.version      { let _ = writeln!(out, "  Version      : {}", v); }
            if let Some(v) = &b.release_date { let _ = writeln!(out, "  Release Date : {}", v); }
        }
    }

    if want("battery") {
        if let Some(b) = &r.battery {
            sep(&mut out); 
            let _ = writeln!(out, "BATTERY");
            if let Some(v) = b.charge_percent { let _ = writeln!(out, "  Charge       : {}%", v); }
            if let Some(v) = &b.status        { let _ = writeln!(out, "  Status       : {}", v); }
        }
    }

    if want("services") {
        if let Some(services) = &r.services {
            sep(&mut out); 
            let _ = writeln!(out, "SERVICES (Running & Auto only)");
            for s in services.iter().filter(|s| s.state == "Running" || s.start_mode == "Auto") {
                let _ = writeln!(out, "  [{:10}] {:<30} {}", s.state, s.start_mode, s.display_name);
            }
        }
    }

    if want("software") {
        sep(&mut out); 
        let _ = writeln!(out, "INSTALLED SOFTWARE");
        for app in &r.software {
            let ver = app.version.as_deref().unwrap_or("?");
            let _ = writeln!(out, "  {} (v{})", app.name, ver);
        }
    }

    sep(&mut out);
    out
}

fn fmt_uptime(secs: u64) -> String {
    let d = secs / 86400; let h = (secs % 86400) / 3600; let m = (secs % 3600) / 60;
    format!("{}d {}h {}m", d, h, m)
}