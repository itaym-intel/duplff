use crate::format::human_bytes;
use duplff_core::actions;
use duplff_core::models::ScanConfig;
use duplff_core::progress::NoopProgress;
use std::io;

/// Run in JSON mode: scan and print the report as JSON to stdout.
pub fn run_json(config: &ScanConfig) -> Result<(), Box<dyn std::error::Error>> {
    let report = duplff_core::find_duplicates(config, &NoopProgress)?;
    let json = serde_json::to_string_pretty(&report)?;
    println!("{json}");
    Ok(())
}

/// Run in dry-run mode: scan and print a human-readable deletion plan.
pub fn run_dry_run(config: &ScanConfig) -> Result<(), Box<dyn std::error::Error>> {
    let report = duplff_core::find_duplicates(config, &NoopProgress)?;

    if report.groups.is_empty() {
        println!("No duplicates found.");
        return Ok(());
    }

    println!(
        "Found {} duplicate group{} ({} files, {} wasted)\n",
        report.groups.len(),
        if report.groups.len() == 1 { "" } else { "s" },
        report.total_files_scanned,
        human_bytes(report.total_wasted_bytes),
    );

    for (i, group) in report.groups.iter().enumerate() {
        let file_count = 1 + group.duplicates.len();
        println!(
            "Group {} ({} files, {} each):",
            i + 1,
            file_count,
            human_bytes(group.size),
        );
        println!(
            "  [KEEP] {} ({})",
            group.keep.entry.path.display(),
            group.keep.reason,
        );
        for dup in &group.duplicates {
            println!("  [DEL]  {}", dup.entry.path.display());
        }
        println!();
    }

    let plan = actions::dry_run(&report.groups);
    println!(
        "Total: {} file{} to delete, {} to reclaim",
        plan.files_to_delete.len(),
        if plan.files_to_delete.len() == 1 {
            ""
        } else {
            "s"
        },
        human_bytes(plan.bytes_to_reclaim),
    );

    Ok(())
}

/// Run in CSV mode: scan and print a CSV report to stdout.
pub fn run_csv(config: &ScanConfig) -> Result<(), Box<dyn std::error::Error>> {
    let report = duplff_core::find_duplicates(config, &NoopProgress)?;

    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(["group_hash", "file_path", "size", "status", "keep_reason"])?;

    for group in &report.groups {
        let hash = hex::encode(group.hash);

        // Keep file
        wtr.write_record([
            &hash,
            &group.keep.entry.path.display().to_string(),
            &group.keep.entry.size.to_string(),
            "keep",
            &group.keep.reason.to_string(),
        ])?;

        // Duplicates
        for dup in &group.duplicates {
            wtr.write_record([
                &hash,
                &dup.entry.path.display().to_string(),
                &dup.entry.size.to_string(),
                "duplicate",
                "",
            ])?;
        }
    }

    wtr.flush()?;
    Ok(())
}
