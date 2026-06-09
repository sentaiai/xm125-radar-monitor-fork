//! Continuous monitoring functions
//!
//! This module handles continuous measurement operations for both distance and presence
//! detection, including CSV export and FIFO output integration.

use crate::cli::Cli;
use crate::display::{
    display_distance_result, display_presence_result, presence_signal_metrics,
    write_distance_to_fifo, write_presence_to_fifo,
};
use crate::error::RadarError;
use crate::fifo::FifoWriter;
use crate::radar::{PresenceMeasurement, XM125Radar};
use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use std::fs::File;
use tokio::time::{sleep, Duration};

/// Monitor distance detection continuously
pub async fn monitor_distance_continuous(
    radar: &mut XM125Radar,
    cli: &Cli,
    count: Option<u32>,
    interval: u64,
    save_to: Option<&str>,
    mut fifo_writer: Option<&mut FifoWriter>,
) -> Result<(), RadarError> {
    let total_measurements = count.unwrap_or(u32::MAX);
    let mut measurement_count = 0u32;

    // Setup progress bar
    let progress = if !cli.output.quiet && count.is_some() {
        let pb = ProgressBar::new(u64::from(total_measurements));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    // Setup CSV writer if requested
    let mut csv_writer = if let Some(filename) = save_to {
        let file = File::create(filename).map_err(|e| RadarError::DeviceError {
            message: format!("Failed to create CSV file: {e}"),
        })?;
        let mut writer = csv::Writer::from_writer(file);

        // Write CSV header
        writer
            .write_record([
                "timestamp",
                "distance_m",
                "signal_strength",
                "temperature_c",
            ])
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to write CSV header: {e}"),
            })?;
        Some(writer)
    } else {
        None
    };

    info!("🚀 Starting continuous distance monitoring...");
    if let Some(count) = count {
        info!("📊 Taking {count} measurements every {interval}ms");
    } else {
        info!("📊 Continuous monitoring every {interval}ms (Ctrl+C to stop)");
    }

    while measurement_count < total_measurements {
        let result = radar.measure_distance().await?;
        let timestamp_full = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();

        // Display result unless quiet mode
        if !cli.output.quiet {
            display_distance_result(&result, &cli.output.format);
        }

        // CSV output
        if let Some(ref mut writer) = csv_writer {
            writer
                .write_record([
                    &timestamp_full,
                    &format!("{:.3}", result.distance),
                    &format!("{:.1}", result.strength),
                    &format!("{:.1}", result.temperature),
                ])
                .map_err(|e| RadarError::DeviceError {
                    message: format!("Failed to write CSV record: {e}"),
                })?;
            writer.flush().map_err(|e| RadarError::DeviceError {
                message: format!("Failed to flush CSV writer: {e}"),
            })?;
        }

        // FIFO output
        if let Some(ref mut writer) = fifo_writer {
            write_distance_to_fifo(writer, &result, &cli.output.fifo_format);
        }

        measurement_count += 1;

        // Update progress bar
        if let Some(ref pb) = progress {
            pb.set_position(u64::from(measurement_count));
        }

        // Break if we've reached the count
        if count.is_some() && measurement_count >= total_measurements {
            break;
        }

        // Wait for next measurement
        sleep(Duration::from_millis(interval)).await;
    }

    // Finish progress bar
    if let Some(pb) = progress {
        pb.finish_with_message("✅ Distance monitoring completed");
    }

    // Print summary
    if let Some(filename) = save_to {
        println!("💾 Results saved to: {filename}");
    }

    Ok(())
}

/// Setup progress bar for monitoring operations
fn setup_progress_bar(cli: &Cli, count: Option<u32>) -> Option<ProgressBar> {
    if !cli.output.quiet && count.is_some() {
        let total_measurements = count.unwrap_or(u32::MAX);
        let pb = ProgressBar::new(u64::from(total_measurements));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    }
}

/// Setup CSV writer for presence monitoring
fn setup_presence_csv_writer(filename: &str) -> Result<csv::Writer<File>, RadarError> {
    let file = File::create(filename).map_err(|e| RadarError::DeviceError {
        message: format!("Failed to create CSV file: {e}"),
    })?;
    let mut writer = csv::Writer::from_writer(file);

    // Write CSV header
    writer
        .write_record([
            "timestamp",
            "measurement_number",
            "presence_detected",
            "presence_distance_m",
            "intra_score",
            "inter_score",
            "signal_quality",
            "confidence",
        ])
        .map_err(|e| RadarError::DeviceError {
            message: format!("Failed to write CSV header: {e}"),
        })?;

    Ok(writer)
}


/// Process a single presence measurement (display, CSV, FIFO output)
fn process_presence_measurement(
    result: &PresenceMeasurement,
    measurement_count: u32,
    timestamp: &str,
    cli: &Cli,
    csv_writer: &mut Option<csv::Writer<File>>,
    fifo_writer: &mut Option<&mut FifoWriter>,
) -> Result<(), RadarError> {
    // Display result unless quiet mode
    if !cli.output.quiet {
        display_presence_result(result, &cli.output.format);
    }

    // CSV output
    if let Some(ref mut writer) = csv_writer {
        let (signal_quality, confidence) = presence_signal_metrics(result);

        writer
            .write_record([
                timestamp,
                &measurement_count.to_string(),
                &result.presence_detected.to_string(),
                &format!("{:.3}", result.presence_distance),
                &format!("{:.2}", result.intra_presence_score),
                &format!("{:.2}", result.inter_presence_score),
                signal_quality,
                confidence,
            ])
            .map_err(|e| RadarError::DeviceError {
                message: format!("Failed to write CSV record: {e}"),
            })?;
        writer.flush().map_err(|e| RadarError::DeviceError {
            message: format!("Failed to flush CSV writer: {e}"),
        })?;
    }

    // FIFO output
    if let Some(ref mut writer) = fifo_writer {
        write_presence_to_fifo(writer, result, &cli.output.fifo_format);
    }

    Ok(())
}

/// Monitor presence detection continuously
pub async fn monitor_presence_continuous(
    radar: &mut XM125Radar,
    cli: &Cli,
    count: Option<u32>,
    interval: u64,
    save_to: Option<&str>,
    mut fifo_writer: Option<&mut FifoWriter>,
) -> Result<(), RadarError> {
    let total_measurements = count.unwrap_or(u32::MAX);
    let mut measurement_count = 0u32;

    // Setup components
    let progress = setup_progress_bar(cli, count);
    let mut csv_writer = if let Some(filename) = save_to {
        Some(setup_presence_csv_writer(filename)?)
    } else {
        None
    };

    // Log startup info
    info!("🚀 Starting continuous presence monitoring...");
    if let Some(count) = count {
        info!("📊 Taking {count} measurements every {interval}ms");
    } else {
        info!("📊 Continuous monitoring every {interval}ms (Ctrl+C to stop)");
    }

    // Main monitoring loop
    while measurement_count < total_measurements {
        let result = radar.measure_presence().await?;
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        measurement_count += 1;

        // Process the measurement (display, CSV, FIFO)
        process_presence_measurement(
            &result,
            measurement_count,
            &timestamp,
            cli,
            &mut csv_writer,
            &mut fifo_writer,
        )?;

        // Update progress bar
        if let Some(ref pb) = progress {
            pb.set_position(u64::from(measurement_count));
        }

        // Check if we should stop
        if count.is_some() && measurement_count >= total_measurements {
            break;
        }

        // Wait for next measurement
        sleep(Duration::from_millis(interval)).await;
    }

    // Cleanup and summary
    if let Some(pb) = progress {
        pb.finish_with_message("✅ Presence monitoring completed");
    }

    if let Some(filename) = save_to {
        println!("💾 Results saved to: {filename}");
    }

    Ok(())
}
