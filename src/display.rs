//! Display and output formatting functions
//!
//! This module handles all output formatting and display logic for measurements,
//! including console output and FIFO writing for system integration.

use crate::cli::OutputFormat;
use crate::fifo::{FifoFormat, FifoWriter};
use crate::radar::{DistanceMeasurement, PresenceMeasurement};
use chrono::Utc;

pub fn presence_signal_metrics(result: &PresenceMeasurement) -> (&'static str, &'static str) {
    let max_score = result
        .intra_presence_score
        .max(result.inter_presence_score);

    let signal_quality = if max_score > 2.0 {
        "STRONG"
    } else if max_score > 1.0 {
        "MEDIUM"
    } else if max_score > 0.5 {
        "WEAK"
    } else {
        "NONE"
    };

    let confidence = if result.presence_detected {
        if max_score > 3.0 {
            "HIGH"
        } else if max_score > 1.5 {
            "MEDIUM"
        } else {
            "LOW"
        }
    } else {
        "NONE"
    };

    (signal_quality, confidence)
}

fn presence_movement_type(result: &PresenceMeasurement) -> &'static str {
    if !result.presence_detected {
        "none"
    } else if result.intra_presence_score > result.inter_presence_score {
        "fast"
    } else {
        "slow"
    }
}

fn presence_distance_zone(distance_m: f32, start_m: f32, end_m: f32) -> &'static str {
    if distance_m < start_m || distance_m > end_m {
        "out_of_range"
    } else if distance_m < 1.0 {
        "near"
    } else if distance_m < 3.0 {
        "mid"
    } else {
        "far"
    }
}

fn build_presence_fifo_json(result: &PresenceMeasurement) -> String {
    let (signal_quality, confidence) = presence_signal_metrics(result);

    format!(
        concat!(
            "{{\"confidence\":\"{confidence}\",",
            "\"detection_mode\":\"presence\",",
            "\"intra_score\":{intra_score:.3},",
            "\"inter_score\":{inter_score:.3},",
            "\"presence_detected\":{presence_detected},",
            "\"presence_distance_m\":{presence_distance_m:.3},",
            "\"sensor_type\":\"XM125\",",
            "\"signal_quality\":\"{signal_quality}\",",
            "\"timestamp\":\"{timestamp}\",",
            "\"presence_sticky\":{presence_sticky},",
            //"\"R_start_m\":{start_m:.3},",
            //"\"R_end_m\":{end_m:.3},",
            "\"distance_zone\":\"{distance_zone}\",",
            //"\"intra_threshold\":{intra_threshold:.3},",
            //"\"inter_threshold\":{inter_threshold:.3},",
            "\"actual_frame_rate_hz\":{actual_frame_rate_hz:.3},",
            "\"detector_error\":{detector_error},",
            "\"internal_temperature_c\":{internal_temperature_c},",
            "\"movement_type\":\"{movement_type}\"}}"
        ),
        confidence = confidence,
        intra_score = result.intra_presence_score,
        inter_score = result.inter_presence_score,
        presence_detected = result.presence_detected,
        presence_distance_m = result.presence_distance,
        signal_quality = signal_quality,
        timestamp = result.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
        presence_sticky = result.presence_sticky,
        // start_m = result.start_m,
        // end_m = result.end_m,
        distance_zone =
            presence_distance_zone(result.presence_distance, result.start_m, result.end_m),
        // intra_threshold = result.intra_threshold,
        // inter_threshold = result.inter_threshold,
        actual_frame_rate_hz = result.actual_frame_rate_hz,
        detector_error = result.detector_error,
        internal_temperature_c = result.internal_temperature_c,
        movement_type = presence_movement_type(result),
    )
}

/// Display distance measurement result in the specified format
pub fn display_distance_result(result: &DistanceMeasurement, format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            let json_result = serde_json::json!({
                "timestamp": Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                "distance_m": result.distance,
                "signal_strength": result.strength,
                "temperature_c": result.temperature
            });
            println!("{}", serde_json::to_string_pretty(&json_result).unwrap());
        }
        OutputFormat::Csv => {
            println!("timestamp,distance_m,signal_strength,temperature_c");
            println!(
                "{},{:.3},{:.1},{:.1}",
                Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                result.distance,
                result.strength,
                result.temperature
            );
        }
        OutputFormat::Human => {
            println!(
                "📏 Distance: {:.3}m | Signal: {:.1} | Temp: {:.1}°C",
                result.distance, result.strength, result.temperature
            );
        }
    }
}

/// Display presence measurement result in the specified format
pub fn display_presence_result(result: &PresenceMeasurement, format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            let (signal_quality, confidence) = presence_signal_metrics(result);
            let json_result = serde_json::json!({
                "timestamp": result.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                "presence_detected": result.presence_detected,
                "presence_distance_m": result.presence_distance,
                "intra_score": result.intra_presence_score,
                "inter_score": result.inter_presence_score,
                "signal_quality": signal_quality,
                "confidence": confidence,
            });
            println!("{}", serde_json::to_string_pretty(&json_result).unwrap());
        }
        OutputFormat::Csv => {
            let (signal_quality, confidence) = presence_signal_metrics(result);
            println!("timestamp,presence_detected,presence_distance_m,intra_score,inter_score,signal_quality,confidence");
            println!(
                "{},{},{:.3},{:.2},{:.2},{},{}",
                result.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                result.presence_detected,
                result.presence_distance,
                result.intra_presence_score,
                result.inter_presence_score,
                signal_quality,
                confidence
            );
        }
        OutputFormat::Human => {
            let status = if result.presence_detected {
                "🟢 DETECTED"
            } else {
                "🔴 NONE"
            };
            let (_, confidence) = presence_signal_metrics(result);

            println!(
                "{} | Distance: {:.3}m | Scores: {:.2}/{:.2} | Confidence: {}",
                status,
                result.presence_distance,
                result.intra_presence_score,
                result.inter_presence_score,
                confidence
            );
        }
    }
}

/// Write distance measurement to FIFO with timing control
pub fn write_distance_to_fifo(
    writer: &mut FifoWriter,
    result: &DistanceMeasurement,
    format: &FifoFormat,
) {
    match format {
        FifoFormat::Simple => {
            // Simple format: presence_state (always 1 for distance) and distance
            let _ = writer.write_timed_simple(1, result.distance);
        }
        FifoFormat::Json => {
            let json_data = serde_json::json!({
                "timestamp": Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                "sensor_type": "XM125",
                "detection_mode": "distance",
                "distance_m": result.distance,
                "signal_strength": result.strength,
                "temperature_c": result.temperature
            });
            let _ = writer.write_timed_json(&json_data);
        }
    }
}

/// Write presence measurement to FIFO with timing control
pub fn write_presence_to_fifo(
    writer: &mut FifoWriter,
    result: &PresenceMeasurement,
    format: &FifoFormat,
) {
    match format {
        FifoFormat::Simple => {
            // BGT60TR13C compatible format: presence_state (0/1) and distance
            let presence_state = i32::from(result.presence_detected);
            let _ = writer.write_timed_simple(presence_state, result.presence_distance);
        }
        FifoFormat::Json => {
            let _ = writer.write_timed_line(&build_presence_fifo_json(result));
        }
    }
}
