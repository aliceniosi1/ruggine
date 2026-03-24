use sysinfo::{CpuExt, System, SystemExt};
use std::fs::OpenOptions;
use std::io::Write;

/// Legge l’utilizzo globale di CPU e lo appende in cpu.log.
/// Restituisce Ok(()) se l’operazione va a buon fine.
pub async fn log_cpu_usage() -> std::io::Result<()> {
    // Crea un oggetto System e aggiorna i dati CPU
    let mut sys = System::new_all();
    sys.refresh_cpu();
    let cpu_usage = sys.global_cpu_info().cpu_usage();

    // Apri (o crea) il file cpu.log in append mode
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("../../cpu.log")?;

    // Scrivi la riga con il valore di CPU
    writeln!(file, "CPU Usage: {:.2}%", cpu_usage)?;
    Ok(())
}