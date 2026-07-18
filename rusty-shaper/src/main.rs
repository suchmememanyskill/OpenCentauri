use std::io::Write;
use std::path::{Path, PathBuf};

use clap::Parser;

use rusty_shaper::{
    Result, ShaperError, ShaperModel,
    input::PsdInput,
    models::{all_shapers_with_zvd, shaper_by_name},
    moonraker::send_gcode_script,
    scorer::{MIN_FREQ, ShaperCalibrator},
};

fn valid_shapers_list() -> String {
    all_shapers_with_zvd()
        .into_iter()
        .map(|s| s.name().to_string())
        .chain(std::iter::once("all".to_string()))
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Parser, Debug)]
#[command(name = "rusty-shaper")]
#[command(about = "Low-RAM Rust input shaper calibration for 3D printers")]
#[command(version)]
struct Args {
    /// Input CSV file (PSD or raw accelerometer data)
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// Output format(s): "cfg", "csv", "json", "json-pretty", "klippy", or file path
    /// Can be specified multiple times for multiple outputs
    #[arg(short, long, value_name = "MODE", action = clap::ArgAction::Append)]
    output: Vec<String>,

    /// Working directory for output files (default: /tmp)
    #[arg(long, value_name = "DIR", default_value = "/tmp")]
    workdir: PathBuf,

    /// Name suffix for output files (default: timestamp %Y%m%d_%H%M%S)
    #[arg(short, long, value_name = "NAME")]
    name: Option<String>,

    /// Shaper types to test (comma-separated, e.g. "zv,mzv,ei")
    /// Default matches Kalico's AUTOTUNE_SHAPERS (excludes ZVD).
    /// Use "zvd" explicitly or "all" to include ZVD.
    #[arg(short, long, default_value = "zv,mzv,ei,2hump_ei,3hump_ei")]
    shapers: String,

    /// Damping ratio
    #[arg(long, default_value = "0.1")]
    damping_ratio: f64,

    /// Test damping ratios (comma-separated)
    #[arg(long, default_value = "0.075,0.1,0.15")]
    test_damping_ratios: String,

    /// Square corner velocity (mm/s)
    #[arg(long, default_value = "5.0")]
    scv: f64,

    /// Maximum allowed smoothing (mm)
    #[arg(short, long)]
    max_smoothing: Option<f64>,

    /// Maximum frequency to analyze (Hz)
    #[arg(long, default_value = "200.0")]
    max_freq: f64,

    /// Shaper frequency range: start:end:step
    #[arg(long, value_name = "RANGE")]
    shaper_freq: Option<String>,

    /// Parse input as raw accelerometer data (not PSD)
    /// Only needed if the file lacks the standard #time,accel_x,accel_y,accel_z header
    #[arg(long)]
    raw: bool,

    /// Window time for PSD computation (seconds)
    #[arg(long, default_value = "0.5")]
    window_t: f64,

    /// Include PSD bins in JSON output (default: omitted to keep files small)
    #[arg(long)]
    with_psd: bool,

    /// Suppress non-error output
    #[arg(short, long)]
    quiet: bool,

    /// Moonraker IP address (default: 127.0.0.1)
    #[arg(long, value_name = "IP", default_value = "127.0.0.1")]
    moonraker_ip: String,

    /// Moonraker port (default: 80)
    #[arg(long, value_name = "PORT", default_value = "80")]
    moonraker_port: u16,

    /// Commit shaper params to printer.cfg (sends SAVE_CONFIG RESTART=0)
    #[arg(long)]
    commit: bool,
}

fn main() -> Result<()> {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<()> {
    let args = Args::parse();
    validate_args(&args)?;

    let test_damping_ratios = parse_damping_ratios(&args.test_damping_ratios)?;
    let shapers = parse_shaper_selection(&args.shapers)?;

    let mut calibrator = ShaperCalibrator::new()
        .with_damping_ratio(args.damping_ratio)
        .with_test_damping_ratios(test_damping_ratios)
        .with_scv(args.scv)
        .with_max_freq(args.max_freq);

    if let Some(max_sm) = args.max_smoothing {
        calibrator = calibrator.with_max_smoothing(max_sm);
    }

    if let Some(ref freq_str) = args.shaper_freq {
        let (start, end, step) = parse_freq_range(freq_str)?;
        calibrator = calibrator.with_freq_range(start, end, step);
    }

    for shaper in shapers {
        calibrator = calibrator.with_shaper(shaper);
    }

    let mut psd = if args.raw {
        PsdInput::from_raw_csv_streaming(&args.input, args.window_t)?
    } else {
        PsdInput::from_csv_with_window(&args.input, args.window_t)?
    };

    psd.normalize();
    psd.suppress_low_freq(MIN_FREQ);

    let result = calibrator.fit(&psd)?;
    let axis = axis_from_path(&args.input);
    let name_suffix = args
        .name
        .clone()
        .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());

    for output_mode in &args.output {
        match output_mode.as_str() {
            "klippy" => apply_via_klippy(&args, axis, &result)?,
            "cfg" => print_cfg_block(axis, &result),
            "csv" => write_csv_output(&args, axis, &result, &name_suffix)?,
            "json" => write_json_file(&args, axis, &result, &name_suffix, false)?,
            "json-pretty" => write_json_file(&args, axis, &result, &name_suffix, true)?,
            path => write_json_path(&args, path, &result)?,
        }
    }

    if !args.quiet {
        print_summary_stderr(&result);
    }
    Ok(())
}

fn validate_args(args: &Args) -> Result<()> {
    validate_damping_ratio(args.damping_ratio, "--damping-ratio")?;
    validate_non_negative(args.scv, "--scv")?;
    validate_positive(args.max_freq, "--max-freq")?;
    validate_positive(args.window_t, "--window-t")?;

    if let Some(max_smoothing) = args.max_smoothing {
        validate_non_negative(max_smoothing, "--max-smoothing")?;
    }

    if args.commit && !args.output.iter().any(|mode| mode == "klippy") {
        return Err(ShaperError::Cli(
            "--commit requires --output klippy".to_string(),
        ));
    }

    Ok(())
}

fn validate_damping_ratio(value: f64, option: &str) -> Result<()> {
    if value.is_finite() && (0.0..1.0).contains(&value) {
        return Ok(());
    }

    Err(ShaperError::Cli(format!(
        "{option} must be finite and in the range [0.0, 1.0)"
    )))
}

fn validate_positive(value: f64, option: &str) -> Result<()> {
    if value.is_finite() && value > 0.0 {
        return Ok(());
    }

    Err(ShaperError::Cli(format!("{option} must be finite and > 0")))
}

fn validate_non_negative(value: f64, option: &str) -> Result<()> {
    if value.is_finite() && value >= 0.0 {
        return Ok(());
    }

    Err(ShaperError::Cli(format!(
        "{option} must be finite and >= 0"
    )))
}

fn parse_damping_ratios(value: &str) -> Result<Vec<f64>> {
    let ratios: Vec<f64> = value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| part.parse::<f64>())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| ShaperError::Cli(format!("Invalid --test-damping-ratios: {e}")))?;

    if ratios.is_empty() {
        return Err(ShaperError::Cli(
            "--test-damping-ratios must include at least one value".to_string(),
        ));
    }

    for ratio in &ratios {
        validate_damping_ratio(*ratio, "--test-damping-ratios")?;
    }

    Ok(ratios)
}

fn parse_freq_range(value: &str) -> Result<(f64, f64, f64)> {
    let parts: Vec<f64> = value
        .split(':')
        .map(str::trim)
        .map(|part| part.parse::<f64>())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| ShaperError::Cli(format!("Invalid --shaper-freq range: {e}")))?;

    if parts.len() != 3 {
        return Err(ShaperError::Cli(
            "--shaper-freq must be start:end:step".to_string(),
        ));
    }

    validate_positive(parts[0], "--shaper-freq start")?;
    validate_positive(parts[1], "--shaper-freq end")?;
    validate_positive(parts[2], "--shaper-freq step")?;

    if parts[0] >= parts[1] {
        return Err(ShaperError::Cli(
            "--shaper-freq start must be less than end".to_string(),
        ));
    }

    Ok((parts[0], parts[1], parts[2]))
}

fn parse_shaper_selection(selection: &str) -> Result<Vec<Box<dyn ShaperModel>>> {
    let names: Vec<String> = selection
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();

    if names.is_empty() {
        return Err(ShaperError::Cli(format!(
            "--shapers must include at least one shaper ({})",
            valid_shapers_list()
        )));
    }

    if names.iter().any(|name| name == "all") {
        if names.len() != 1 {
            return Err(ShaperError::Cli(
                "--shapers all cannot be combined with other shapers".to_string(),
            ));
        }
        return Ok(all_shapers_with_zvd());
    }

    let mut seen = Vec::new();
    let mut shapers = Vec::new();

    for name in names {
        if seen.iter().any(|seen_name| seen_name == &name) {
            continue;
        }

        let shaper = shaper_by_name(&name).ok_or_else(|| {
            ShaperError::Cli(format!(
                "Unknown shaper '{name}'. Valid values: {}",
                valid_shapers_list()
            ))
        })?;
        seen.push(name);
        shapers.push(shaper);
    }

    Ok(shapers)
}

fn axis_from_path(path: &Path) -> &'static str {
    if path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| stem.contains("_y") || stem.contains("-y"))
    {
        "y"
    } else {
        "x"
    }
}

fn apply_via_klippy(
    args: &Args,
    axis: &str,
    result: &rusty_shaper::types::CalibrationOutput,
) -> Result<()> {
    let axis_up = axis.to_uppercase();

    for fit in &result.all_results {
        let m118 = format!(
            "M118 Fitted shaper '{}' frequency = {:.1} Hz (vibrations = {:.1}%, smoothing ~= {:.3})\n\
             M118 To avoid too much smoothing with '{}', suggested max_accel <= {:.0} mm/sec^2",
            fit.shaper_name,
            fit.best.freq,
            fit.best.vibrs * 100.0,
            fit.best.smoothing,
            fit.shaper_name,
            (fit.best.max_accel / 100.0).round() * 100.0
        );
        if let Err(e) = send_gcode_script(&args.moonraker_ip, args.moonraker_port, &m118) {
            eprintln!("Warning: failed to send M118: {e}");
        }
    }

    let m118_rec = format!(
        "M118 Recommended shaper_type_{} = {}, shaper_freq_{} = {:.1} Hz",
        axis, result.recommended_shaper, axis, result.recommended_freq
    );
    if let Err(e) = send_gcode_script(&args.moonraker_ip, args.moonraker_port, &m118_rec) {
        eprintln!("Warning: failed to send M118: {e}");
    }

    let script = format!(
        "SET_INPUT_SHAPER SHAPER_TYPE_{}={} SHAPER_FREQ_{}={:.1}",
        axis_up,
        result.recommended_shaper.to_uppercase(),
        axis_up,
        result.recommended_freq
    );
    send_gcode_script(&args.moonraker_ip, args.moonraker_port, &script)?;

    if !args.quiet {
        eprintln!(
            "✓ Set input_shaper {axis_up} = {} @ {:.1} Hz on Klippy",
            result.recommended_shaper.to_uppercase(),
            result.recommended_freq
        );
    }

    if args.commit {
        send_gcode_script(
            &args.moonraker_ip,
            args.moonraker_port,
            "SAVE_CONFIG RESTART=0",
        )?;
        if !args.quiet {
            eprintln!("✓ Config saved (SAVE_CONFIG RESTART=0)");
        }
    } else if !args.quiet {
        eprintln!("Note: Parameters are in RAM only. Use --commit to persist to printer.cfg.");
    }
    Ok(())
}

fn print_cfg_block(axis: &str, result: &rusty_shaper::types::CalibrationOutput) {
    let max_accel = result.recommended_max_accel;
    println!("[input_shaper]");
    println!(
        "# Recommended max_accel <= {:.0} mm/s²",
        (max_accel / 100.0).round() * 100.0
    );
    println!(
        "shaper_type_{} = {}",
        axis,
        result.recommended_shaper.to_lowercase()
    );
    println!("shaper_freq_{} = {:.1}", axis, result.recommended_freq);
}

fn write_json_file(
    args: &Args,
    axis: &str,
    result: &rusty_shaper::types::CalibrationOutput,
    name_suffix: &str,
    pretty: bool,
) -> Result<()> {
    std::fs::create_dir_all(&args.workdir)?;
    let path = args
        .workdir
        .join(format!("calibration_data_{axis}_{name_suffix}.json"));
    let json = serialize_output(result, args.with_psd, pretty)?;
    std::fs::write(&path, json)?;
    if !args.quiet {
        eprintln!("JSON written to {}", path.display());
    }
    Ok(())
}

fn write_json_path(
    args: &Args,
    path: &str,
    result: &rusty_shaper::types::CalibrationOutput,
) -> Result<()> {
    let path = Path::new(path);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }

    let json = serialize_output(result, args.with_psd, false)?;
    std::fs::write(path, json)?;
    if !args.quiet {
        eprintln!("Results written to {}", path.display());
    }
    Ok(())
}

fn serialize_output(
    result: &rusty_shaper::types::CalibrationOutput,
    with_psd: bool,
    pretty: bool,
) -> Result<String> {
    if with_psd {
        return if pretty {
            Ok(serde_json::to_string_pretty(result)?)
        } else {
            Ok(serde_json::to_string(result)?)
        };
    }

    let mut value = serde_json::to_value(result).map_err(ShaperError::Json)?;
    if let serde_json::Value::Object(ref mut map) = value {
        map.remove("psd_bins");
    }

    if pretty {
        Ok(serde_json::to_string_pretty(&value)?)
    } else {
        Ok(serde_json::to_string(&value)?)
    }
}

fn write_csv_output(
    args: &Args,
    axis: &str,
    result: &rusty_shaper::types::CalibrationOutput,
    name_suffix: &str,
) -> Result<()> {
    std::fs::create_dir_all(&args.workdir)?;
    let path = args
        .workdir
        .join(format!("calibration_data_{axis}_{name_suffix}.csv"));
    let file = std::fs::File::create(&path)?;
    let mut writer = std::io::BufWriter::new(file);

    let mut header = "freq,psd_x,psd_y,psd_z,psd_xyz".to_string();
    for fit in &result.all_results {
        header.push_str(&format!(",{}({:.1})", fit.shaper_name, fit.best.freq));
    }
    writeln!(writer, "{header}")?;

    let shaper_coeffs: Vec<_> = result
        .all_results
        .iter()
        .map(|fit| {
            rusty_shaper::models::get_shaper_coefficients(
                &fit.shaper_name,
                fit.best.freq,
                args.damping_ratio,
            )
            .ok_or_else(|| {
                ShaperError::Cli(format!(
                    "Internal error: no coefficients for shaper {}",
                    fit.shaper_name
                ))
            })
        })
        .collect::<Result<Vec<_>>>()?;

    for bin in &result.psd_bins {
        let mut row = format!(
            "{:.1},{:.6e},{:.6e},{:.6e},{:.6e}",
            bin.freq, bin.psd_x, bin.psd_y, bin.psd_z, bin.psd_sum
        );
        for coeffs in &shaper_coeffs {
            let response =
                rusty_shaper::scorer::estimate_shaper(coeffs, args.damping_ratio, &[bin.freq])[0];
            row.push_str(&format!(",{:.6e}", response));
        }
        writeln!(writer, "{row}")?;
    }

    if !args.quiet {
        eprintln!("CSV written to {}", path.display());
    }
    Ok(())
}

fn print_summary_stderr(result: &rusty_shaper::types::CalibrationOutput) {
    eprintln!(
        "Recommended shaper: {} @ {:.1} Hz",
        result.recommended_shaper.to_uppercase(),
        result.recommended_freq
    );
    for fit in &result.all_results {
        eprintln!(
            "  {}: best={:.1} Hz, vibr={:.1}%, smoothing={:.3}, score={:.4}, max_accel<={:.0}",
            fit.shaper_name.to_uppercase(),
            fit.best.freq,
            fit.best.vibrs * 100.0,
            fit.best.smoothing,
            fit.best.score,
            (fit.best.max_accel / 100.0).round() * 100.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_args() -> Args {
        Args {
            input: PathBuf::from("input.csv"),
            output: Vec::new(),
            workdir: PathBuf::from("/tmp"),
            name: None,
            shapers: "mzv".to_string(),
            damping_ratio: 0.1,
            test_damping_ratios: "0.075,0.1,0.15".to_string(),
            scv: 5.0,
            max_smoothing: None,
            max_freq: 200.0,
            shaper_freq: None,
            raw: false,
            window_t: 0.5,
            with_psd: false,
            quiet: false,
            moonraker_ip: "127.0.0.1".to_string(),
            moonraker_port: 80,
            commit: false,
        }
    }

    #[test]
    fn parse_shapers_all_includes_zvd() {
        let shapers = parse_shaper_selection("all").unwrap();
        assert!(shapers.iter().any(|shaper| shaper.name() == "zvd"));
    }

    #[test]
    fn parse_shapers_rejects_unknown_names() {
        let err = parse_shaper_selection("mzv,not_real").err().unwrap();
        assert!(err.to_string().contains("Unknown shaper"));
    }

    #[test]
    fn parse_shapers_rejects_combined_all() {
        let err = parse_shaper_selection("all,mzv").err().unwrap();
        assert!(err.to_string().contains("cannot be combined"));
    }

    #[test]
    fn parse_freq_range_rejects_bad_bounds() {
        let err = parse_freq_range("60:20:0.5").unwrap_err();
        assert!(err.to_string().contains("start must be less than end"));
    }

    #[test]
    fn commit_requires_klippy_output() {
        let mut args = test_args();
        args.commit = true;

        let err = validate_args(&args).err().unwrap();
        assert!(err.to_string().contains("--output klippy"));

        args.output.push("klippy".to_string());
        validate_args(&args).unwrap();
    }

    #[test]
    fn json_output_omits_psd_bins_by_default() {
        let result = rusty_shaper::types::CalibrationOutput {
            recommended_shaper: "mzv".to_string(),
            recommended_freq: 40.0,
            recommended_max_accel: 10000.0,
            all_results: Vec::new(),
            psd_bins: vec![rusty_shaper::types::PsdBin::new(10.0, 1.0, 0.0, 0.0)],
        };

        let json = serialize_output(&result, false, false).expect("serialize should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(!parsed.as_object().unwrap().contains_key("psd_bins"));
    }

    #[test]
    fn json_output_includes_psd_bins_with_flag() {
        let result = rusty_shaper::types::CalibrationOutput {
            recommended_shaper: "mzv".to_string(),
            recommended_freq: 40.0,
            recommended_max_accel: 10000.0,
            all_results: Vec::new(),
            psd_bins: vec![rusty_shaper::types::PsdBin::new(10.0, 1.0, 0.0, 0.0)],
        };

        let json = serialize_output(&result, true, false).expect("serialize should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.as_object().unwrap().contains_key("psd_bins"));
    }
}
