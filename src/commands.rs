//! Command execution logic
//!
//! This module handles the main command dispatch and execution logic,
//! coordinating between different measurement modes and output formats.

use crate::cli::{Cli, Commands, OutputFormat};
use crate::config::{
    configure_distance_range, configure_presence_parameters, debug_registers_if_connected,
};
use crate::display::{
    display_distance_result, display_presence_result, write_distance_to_fifo,
    write_presence_to_fifo,
};
use crate::error::RadarError;
use crate::fifo::FifoWriter;
use crate::handlers::handle_firmware_action;
use crate::monitoring::{monitor_distance_continuous, monitor_presence_continuous};
use crate::radar::{DetectorMode, XM125Radar};
use serde_json::json;

/// Parameters for distance measurement command
struct DistanceParams<'a> {
    range: &'a Option<String>,
    continuous: bool,
    count: Option<u32>,
    interval: u64,
    save_to: &'a Option<String>,
}

/// Parameters for presence detection command
struct PresenceParams<'a> {
    range: &'a Option<crate::cli::PresenceRange>,
    min_range: Option<f32>,
    max_range: Option<f32>,
    sensitivity: Option<f32>,
    frame_rate: Option<f32>,
    profile: &'a crate::cli::ProfileMode,
    continuous: bool,
    count: Option<u32>,
    interval: u64,
    save_to: &'a Option<String>,
}

/// Handle status command output in different formats
fn handle_status_command(status: &str, format: &OutputFormat) -> Result<(), RadarError> {
    match format {
        OutputFormat::Json => {
            let status_obj = json!({ "status": status });
            println!("{}", serde_json::to_string_pretty(&status_obj)?);
        }
        OutputFormat::Csv => {
            println!("status");
            println!("{status}");
        }
        OutputFormat::Human => {
            println!("📡 XM125 Status: {status}");
        }
    }
    Ok(())
}

/// Handle info command output in different formats
fn handle_info_command(info: &str, format: &OutputFormat) -> Result<(), RadarError> {
    match format {
        OutputFormat::Json => {
            let info_obj = json!({ "info": info });
            println!("{}", serde_json::to_string_pretty(&info_obj)?);
        }
        OutputFormat::Csv => {
            println!("info");
            println!("{info}");
        }
        OutputFormat::Human => {
            println!("🔍 XM125 Device Information:");
            println!("{info}");
        }
    }
    Ok(())
}

/// Handle distance measurement command
async fn handle_distance_command(
    radar: &mut XM125Radar,
    cli: &Cli,
    params: DistanceParams<'_>,
    fifo_writer: Option<&mut FifoWriter>,
) -> Result<(), RadarError> {
    // Ensure device is in distance mode
    radar.set_detector_mode(DetectorMode::Distance);

    // Configure range if specified
    if let Some(range_str) = params.range {
        configure_distance_range(radar, range_str)?;
    }

    // Debug registers if requested (global option)
    if cli.logging.debug_registers {
        debug_registers_if_connected(radar, "Distance");
    }

    if params.continuous {
        monitor_distance_continuous(
            radar,
            cli,
            params.count,
            params.interval,
            params.save_to.as_deref(),
            fifo_writer,
        )
        .await?;
    } else {
        let result = radar.measure_distance().await?;
        display_distance_result(&result, &cli.output.format);

        // Single measurement FIFO output
        if let Some(writer) = fifo_writer {
            write_distance_to_fifo(writer, &result, &cli.output.fifo_format);
        }
    }
    Ok(())
}

/// Handle presence detection command
async fn handle_presence_command(
    radar: &mut XM125Radar,
    cli: &Cli,
    params: PresenceParams<'_>,
    fifo_writer: Option<&mut FifoWriter>,
) -> Result<(), RadarError> {
    // Ensure device is in presence mode
    radar.set_detector_mode(DetectorMode::Presence);

    // Configure presence parameters
    configure_presence_parameters(
        radar,
        params.range.as_ref(),
        params.min_range,
        params.max_range,
        params.sensitivity,
        params.frame_rate,
        params.profile,
    )?;

    // Debug registers if requested (global option)
    if cli.logging.debug_registers {
        debug_registers_if_connected(radar, "Presence");
    }

    if params.continuous {
        monitor_presence_continuous(
            radar,
            cli,
            params.count,
            params.interval,
            params.save_to.as_deref(),
            fifo_writer,
        )
        .await?;
    } else {
        let result = radar.measure_presence().await?;
        display_presence_result(&result, &cli.output.format);

        // Single measurement FIFO output
        if let Some(writer) = fifo_writer {
            write_presence_to_fifo(writer, &result, &cli.output.fifo_format);
        }
    }
    Ok(())
}

/// Handle registers command — full presence I2C register snapshot
fn handle_registers_command(
    radar: &mut XM125Radar,
    format: &OutputFormat,
) -> Result<(), RadarError> {
    let snapshot = radar.read_presence_registers()?;

    if snapshot.application_id != crate::radar::PRESENCE_APP_ID {
        return Err(RadarError::DeviceError {
            message: format!(
                "Expected presence firmware (application_id=2), got {}",
                snapshot.application_id
            ),
        });
    }

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&snapshot)?);
        }
        OutputFormat::Human => {
            println!("📋 XM125 Presence Register Snapshot");
            println!("{}", serde_json::to_string_pretty(&snapshot)?);
        }
        OutputFormat::Csv => {
            return Err(RadarError::DeviceError {
                message: "CSV format not supported for registers command; use --format json"
                    .to_string(),
            });
        }
    }
    Ok(())
}

/// Execute the main command logic
pub async fn execute_command(
    cli: &Cli,
    radar: &mut XM125Radar,
    fifo_writer: Option<&mut FifoWriter>,
) -> Result<(), RadarError> {
    match &cli.command {
        Commands::Status => {
            let status = radar.get_status()?;
            handle_status_command(&status, &cli.output.format)?;
        }

        Commands::Info => {
            let info = radar.get_info()?;
            handle_info_command(&info, &cli.output.format)?;
        }

        Commands::Registers => {
            handle_registers_command(radar, &cli.output.format)?;
        }

        Commands::Distance {
            range,
            continuous,
            count,
            interval,
            save_to,
        } => {
            let params = DistanceParams {
                range,
                continuous: *continuous,
                count: *count,
                interval: *interval,
                save_to,
            };
            handle_distance_command(radar, cli, params, fifo_writer).await?;
        }

        Commands::Presence {
            range,
            min_range,
            max_range,
            sensitivity,
            frame_rate,
            profile,
            continuous,
            count,
            interval,
            save_to,
        } => {
            let params = PresenceParams {
                range,
                min_range: *min_range,
                max_range: *max_range,
                sensitivity: *sensitivity,
                frame_rate: *frame_rate,
                profile,
                continuous: *continuous,
                count: *count,
                interval: *interval,
                save_to,
            };
            handle_presence_command(radar, cli, params, fifo_writer).await?;
        }

        Commands::Firmware { action } => {
            handle_firmware_action(radar, action, &cli.firmware_path).await?;
        }

        Commands::Gpio { .. } => {
            // GPIO commands are handled earlier, this should not be reached
            unreachable!("GPIO commands should be handled before I2C initialization");
        }
    }
    Ok(())
}
