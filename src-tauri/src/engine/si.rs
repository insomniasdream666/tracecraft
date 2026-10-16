use crate::model::geometry::Point;
use crate::model::pcb::{PcbBoard, Trace, LayerStack};
use crate::model::id::LayerId;
use crate::engine::impedance;

/// Signal integrity analysis result for a single trace
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SiResult {
    pub trace_length_mm: f64,
    pub impedance_ohms: f64,
    pub propagation_delay_ps: f64,
    pub reflection_coefficient: f64,
    pub return_loss_db: f64,
    pub dominant_freq_ghz: f64,
    pub attenuation_db_per_m: f64,
}

/// Eye diagram data points for visualization
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct EyeDiagramResult {
    pub unit_interval_ps: f64,
    pub eye_height_v: f64,
    pub eye_width_ps: f64,
    pub jitter_rms_ps: f64,
    pub sample_points: Vec<EyeSample>,
    pub mask_violations: usize,
}

/// Single eye diagram sample point
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct EyeSample {
    pub time_ps: f64,
    pub voltage_v: f64,
    pub is_violation: bool,
}

/// Reflection simulation result
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ReflectionResult {
    pub incident_voltage_v: f64,
    pub reflected_voltage_v: f64,
    pub transmitted_voltage_v: f64,
    pub reflection_coefficient: f64,
    pub time_domain_reflections: Vec<ReflectionEvent>,
}

/// Single reflection event in time domain
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ReflectionEvent {
    pub time_ps: f64,
    pub voltage_v: f64,
    pub is_incident: bool,
}

/// Transmission line analysis result
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct TransmissionLineResult {
    pub characteristic_impedance_ohms: f64,
    pub propagation_constant: f64,
    pub phase_velocity_mm_ps: f64,
    pub group_velocity_mm_ps: f64,
    pub wavelength_mm: f64,
    pub attenuation_db_per_m: f64,
    pub frequency_sweep: Vec<FrequencyPoint>,
}

/// Frequency sweep point
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct FrequencyPoint {
    pub frequency_ghz: f64,
    pub impedance_ohms: f64,
    pub phase_deg: f64,
    pub attenuation_db: f64,
}

/// Crosstalk estimation between two parallel trace segments
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CrosstalkResult {
    pub victim_trace: usize,
    pub aggressor_trace: usize,
    pub coupling_length_mm: f64,
    pub near_end_crosstalk_db: f64,
    pub far_end_crosstalk_db: f64,
}

/// Velocity of light in vacuum (mm/ps)
const C0: f64 = 0.299792458; // mm/ps

/// Analyze signal integrity for a specific trace
pub fn analyze_trace(
    trace: &Trace,
    stackup: &LayerStack,
    _board: &PcbBoard,
) -> SiResult {
    let trace_len = trace_length(trace);
    let t = impedance::copper_thickness_mm(stackup.copper_weight_oz);
    let (er, _height) = find_dielectric(stackup);
    let layer_idx = layer_to_index(stackup, trace.layer).unwrap_or(0);
    let z0 = impedance::impedance_from_width(trace.width, stackup, layer_idx, t);
    let vp = C0 / er.sqrt(); // propagation velocity in mm/ps
    let delay = trace_len / vp; // ps
    let z_target = 50.0;
    let ref_coeff = (z0 - z_target) / (z0 + z_target);
    let rl = if ref_coeff.abs() > 1e-9 {
        20.0 * ref_coeff.abs().log10()
    } else {
        -60.0
    };

    // Estimate dominant frequency from trace length (quarter-wave resonance)
    let freq_ghz = if trace_len > 0.01 {
        (vp * 1e3) / (4.0 * trace_len) // GHz (vp in mm/ps, result in GHz)
    } else {
        0.0
    };
    let freq_mhz = freq_ghz * 1000.0;

    // Attenuation: skin effect + dielectric loss (simplified)
    let skin_depth_mm = 0.066 / (freq_mhz.max(1.0)).sqrt(); // Cu skin depth at freq
    let cond_loss = if trace.width > 0.0 {
        8.686 / (trace.width * skin_depth_mm * 5.8e7 * z0)
    } else {
        0.0
    };
    let diel_loss = 0.5 * std::f64::consts::PI * er.sqrt() * stackup.dielectrics.first().map(|d| d.loss_tangent).unwrap_or(0.02);

    SiResult {
        trace_length_mm: trace_len,
        impedance_ohms: z0,
        propagation_delay_ps: delay,
        reflection_coefficient: ref_coeff,
        return_loss_db: rl,
        dominant_freq_ghz: freq_ghz,
        attenuation_db_per_m: cond_loss + diel_loss,
    }
}

/// Find crosstalk between two traces on the same layer
pub fn estimate_crosstalk(
    a: &Trace,
    b: &Trace,
    _stackup: &LayerStack,
) -> Option<CrosstalkResult> {
    if a.layer != b.layer { return None; }
    if a.points.len() < 2 || b.points.len() < 2 { return None; }

    let mut coupling_length = 0.0;
    let mut min_gap = f64::MAX;

    for seg_a in a.points.windows(2) {
        for seg_b in b.points.windows(2) {
            let d = segment_distance(&seg_a[0], &seg_a[1], &seg_b[0], &seg_b[1]);
            let gap = d - a.width / 2.0 - b.width / 2.0;
            if gap < 1.0 { // only consider traces within 1mm
                coupling_length += segment_length(&seg_a[0], &seg_a[1]);
                if gap < min_gap { min_gap = gap; }
            }
        }
    }

    if coupling_length < 0.1 { return None; }

    // Simple crosstalk model: NEXT ~ -20*log10(1 + gap/H)
    let h: f64 = 0.2; // typical dielectric height
    let coupling = 1.0 + (min_gap / h.max(0.01));
    let next_db = -20.0 * coupling.log10();
    let fext_db = next_db - 6.0; // FEXT typically ~6dB lower than NEXT

    Some(CrosstalkResult {
        victim_trace: 0,
        aggressor_trace: 1,
        coupling_length_mm: coupling_length,
        near_end_crosstalk_db: next_db,
        far_end_crosstalk_db: fext_db,
    })
}

/// Analyze all traces and find SI issues
pub fn analyze_board_si(board: &PcbBoard) -> Vec<SiResult> {
    let mut results = Vec::new();
    for (_, trace) in board.traces.iter() {
        let r = analyze_trace(trace, &board.layers, board);
        results.push(r);
    }
    results
}

/// Run crosstalk analysis on all trace pairs
pub fn analyze_crosstalk(board: &PcbBoard) -> Vec<CrosstalkResult> {
    let mut results = Vec::new();
    let traces: Vec<(usize, &Trace)> = board.traces.iter().collect();

    for i in 0..traces.len() {
        for j in (i + 1)..traces.len() {
            if let Some(xt) = estimate_crosstalk(traces[i].1, traces[j].1, &board.layers) {
                results.push(xt);
            }
        }
    }
    results
}

fn trace_length(trace: &Trace) -> f64 {
    let mut len = 0.0;
    for w in trace.points.windows(2) {
        len += segment_length(&w[0], &w[1]);
    }
    len
}

fn segment_length(a: &Point, b: &Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dx * dx + dy * dy).sqrt()
}

fn segment_distance(a1: &Point, a2: &Point, b1: &Point, b2: &Point) -> f64 {
    // Minimum distance between two line segments
    let d1 = point_to_segment_distance(a1, b1, b2);
    let d2 = point_to_segment_distance(a2, b1, b2);
    let d3 = point_to_segment_distance(b1, a1, a2);
    let d4 = point_to_segment_distance(b2, a1, a2);
    d1.min(d2).min(d3).min(d4)
}

fn point_to_segment_distance(p: &Point, a: &Point, b: &Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len2 = dx * dx + dy * dy;
    if len2 < 1e-12 {
        return p.distance(a);
    }
    let t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / len2;
    let t = t.clamp(0.0, 1.0);
    let proj = Point::new(a.x + t * dx, a.y + t * dy);
    p.distance(&proj)
}

fn find_dielectric(stackup: &LayerStack) -> (f64, f64) {
    stackup.dielectrics.first()
        .map(|d| (d.er, d.thickness_mm))
        .unwrap_or((4.5, 1.6))
}

fn layer_to_index(stackup: &LayerStack, layer: LayerId) -> Option<usize> {
    stackup.layers.iter().position(|l| l.id == layer)
}

/// Generate eye diagram for a trace based on its characteristics
pub fn generate_eye_diagram(
    trace: &Trace,
    stackup: &LayerStack,
    bit_rate_gbps: f64,
    samples_per_ui: usize,
) -> EyeDiagramResult {
    let trace_len = trace_length(trace);
    let t = impedance::copper_thickness_mm(stackup.copper_weight_oz);
    let (er, _height) = find_dielectric(stackup);
    let layer_idx = layer_to_index(stackup, trace.layer).unwrap_or(0);
    let _z0 = impedance::impedance_from_width(trace.width, stackup, layer_idx, t);
    let vp = C0 / er.sqrt();
    let delay = trace_len / vp;
    let attenuation = trace.attenuation_db_per_m;

    let ui_ps = 1000.0 / bit_rate_gbps; // Unit interval in ps
    let num_uis = 3.0; // Simulate 3 UIs
    let _total_time_ps = num_uis * ui_ps;
    let num_samples = (num_uis * samples_per_ui as f64) as usize;

    let mut sample_points = Vec::new();
    let mut mask_violations = 0;

    // Eye mask: 0.3 UI width, 0.7 V height (normalized)
    let eye_mask_width = ui_ps * 0.3;
    let eye_mask_height = 0.7; // Normalized voltage

    for i in 0..num_samples {
        let t_ps = (i as f64 / samples_per_ui as f64) * num_uis * ui_ps;
        let _t_ns = t_ps / 1000.0;

        // Generate a simplified PRBS-like signal with ISI
        let bit_idx = (t_ps / ui_ps).floor() as usize;
        let bit_time = t_ps - bit_idx as f64 * ui_ps;
        let bit_progress = bit_time / ui_ps;

        // Ideal step response with exponential settling
        let ideal_voltage = if bit_idx % 2 == 0 { 1.0 } else { 0.0 };
        
        // ISI from previous bits (simplified)
        let isi = calculate_isi(bit_idx, delay, ui_ps, attenuation);
        
        // Add jitter
        let deterministic_jitter = (delay / ui_ps).sin() * 0.05 * ui_ps;
        let random_jitter = fastrand::f64() * 0.02 * ui_ps;
        let total_jitter = deterministic_jitter + random_jitter;
        
        let voltage = ideal_voltage + isi + (random_jitter / ui_ps) * 0.5;
        let time_in_ui = (bit_progress + total_jitter / ui_ps) * ui_ps;
        
        // Check eye mask violation
        let eye_center_v = 0.5;
        let eye_center_t = ui_ps * 0.5;
        let dv = (voltage - eye_center_v).abs();
        let dt = (time_in_ui - eye_center_t).abs();
        let is_violation = dv > eye_mask_height / 2.0 && dt < eye_mask_width / 2.0;
        
        if is_violation {
            mask_violations += 1;
        }

        sample_points.push(EyeSample {
            time_ps: time_in_ui,
            voltage_v: voltage.clamp(0.0, 1.0),
            is_violation,
        });
    }

    // Calculate eye metrics
    let eye_height = 1.0 - 2.0 * calculate_rms_noise(&sample_points);
    let eye_width = ui_ps * (1.0 - calculate_jitter_ratio(&sample_points, ui_ps));
    let jitter_rms = calculate_jitter_rms(&sample_points, ui_ps);

    EyeDiagramResult {
        unit_interval_ps: ui_ps,
        eye_height_v: eye_height.max(0.0),
        eye_width_ps: eye_width.max(0.0),
        jitter_rms_ps: jitter_rms,
        sample_points,
        mask_violations,
    }
}

fn calculate_isi(bit_idx: usize, delay_ps: f64, ui_ps: f64, attenuation_db: f64) -> f64 {
    if bit_idx == 0 { return 0.0; }
    let attenuation_linear = 10.0_f64.powf(-attenuation_db * delay_ps / 1000.0 / 20.0);
    let delay_uis = delay_ps / ui_ps;
    if delay_uis < 1.0 {
        return 0.1 * attenuation_linear * (1.0 - delay_uis);
    }
    0.0
}

fn calculate_rms_noise(samples: &[EyeSample]) -> f64 {
    let mean: f64 = samples.iter().map(|s| s.voltage_v).sum::<f64>() / samples.len() as f64;
    let variance: f64 = samples.iter()
        .map(|s| (s.voltage_v - mean).powi(2))
        .sum::<f64>() / samples.len() as f64;
    variance.sqrt()
}

fn calculate_jitter_ratio(samples: &[EyeSample], ui_ps: f64) -> f64 {
    let threshold_crossings: Vec<f64> = samples.windows(2)
        .filter_map(|w| {
            if (w[0].voltage_v - 0.5) * (w[1].voltage_v - 0.5) < 0.0 {
                Some(w[0].time_ps)
            } else {
                None
            }
        })
        .collect();
    
    if threshold_crossings.len() < 2 { return 0.5; }
    
    let expected_period = ui_ps;
    let periods: Vec<f64> = threshold_crossings.windows(2)
        .map(|w| w[1] - w[0])
        .collect();
    let mean_period: f64 = periods.iter().sum::<f64>() / periods.len() as f64;
    let jitter: f64 = periods.iter()
        .map(|p| (p - mean_period).abs())
        .sum::<f64>() / periods.len() as f64;
    
    (jitter / expected_period).min(0.5)
}

fn calculate_jitter_rms(samples: &[EyeSample], ui_ps: f64) -> f64 {
    let crossings: Vec<f64> = samples.windows(2)
        .filter_map(|w| {
            if (w[0].voltage_v - 0.5) * (w[1].voltage_v - 0.5) < 0.0 {
                Some(w[0].time_ps)
            } else {
                None
            }
        })
        .collect();
    
    if crossings.len() < 3 { return ui_ps * 0.05; }
    
    let periods: Vec<f64> = crossings.windows(2)
        .map(|w| w[1] - w[0])
        .collect();
    let mean: f64 = periods.iter().sum::<f64>() / periods.len() as f64;
    let variance: f64 = periods.iter()
        .map(|p| (p - mean).powi(2))
        .sum::<f64>() / periods.len() as f64;
    variance.sqrt()
}

/// Simulate reflections on a transmission line
pub fn simulate_reflections(
    trace: &Trace,
    stackup: &LayerStack,
    source_impedance_ohms: f64,
    load_impedance_ohms: f64,
    source_voltage_v: f64,
    rise_time_ps: f64,
    simulation_time_ps: f64,
) -> ReflectionResult {
    let trace_len = trace_length(trace);
    let t = impedance::copper_thickness_mm(stackup.copper_weight_oz);
    let (er, _height) = find_dielectric(stackup);
    let layer_idx = layer_to_index(stackup, trace.layer).unwrap_or(0);
    let z0 = impedance::impedance_from_width(trace.width, stackup, layer_idx, t);
    let vp = C0 / er.sqrt();
    let tpd = trace_len / vp; // one-way propagation delay
    let _rise_time_ps = rise_time_ps;

    let gamma_s = (source_impedance_ohms - z0) / (source_impedance_ohms + z0); // Source reflection coeff
    let gamma_l = (load_impedance_ohms - z0) / (load_impedance_ohms + z0); // Load reflection coeff

    let mut events = Vec::new();
    let mut time_ps = 0.0;
    let mut voltage = source_voltage_v * z0 / (source_impedance_ohms + z0);
    let incident_voltage = voltage;
    
    // Initial incident wave
    events.push(ReflectionEvent {
        time_ps: 0.0,
        voltage_v: incident_voltage,
        is_incident: true,
    });
    time_ps += tpd;

    // Simulate multiple reflections (up to 10 bounces or simulation time)
    for _bounce in 0..20 {
        if time_ps > simulation_time_ps { break; }
        
        // Reflection at load
        let reflected_v = voltage * gamma_l;
        events.push(ReflectionEvent {
            time_ps,
            voltage_v: reflected_v,
            is_incident: false,
        });
        voltage += reflected_v;
        time_ps += tpd;

        if time_ps > simulation_time_ps { break; }
        
        // Reflection at source
        let reflected_s = reflected_v * gamma_s;
        events.push(ReflectionEvent {
            time_ps,
            voltage_v: reflected_s,
            is_incident: true,
        });
        voltage += reflected_s;
        time_ps += tpd;

        if reflected_v.abs() < 0.001 && reflected_s.abs() < 0.001 { break; }
    }

    let reflected_voltage = events.iter().filter(|e| !e.is_incident).map(|e| e.voltage_v.abs()).sum();
    let transmitted_voltage = voltage;

    ReflectionResult {
        incident_voltage_v: incident_voltage,
        reflected_voltage_v: reflected_voltage,
        transmitted_voltage_v: transmitted_voltage,
        reflection_coefficient: gamma_l,
        time_domain_reflections: events,
    }
}

/// Analyze transmission line characteristics across frequency
pub fn analyze_transmission_line(
    trace: &Trace,
    stackup: &LayerStack,
    max_freq_ghz: f64,
    num_points: usize,
) -> TransmissionLineResult {
    let trace_len = trace_length(trace);
    let t = impedance::copper_thickness_mm(stackup.copper_weight_oz);
    let (er, _height) = find_dielectric(stackup);
    let layer_idx = layer_to_index(stackup, trace.layer).unwrap_or(0);
    let z0 = impedance::impedance_from_width(trace.width, stackup, layer_idx, t);
    let vp = C0 / er.sqrt();
    let delay = trace_len / vp;

    let beta = 2.0 * std::f64::consts::PI / (vp * 1000.0); // rad/mm per GHz
    let alpha = trace.attenuation_db_per_m / 8.686; // Np/mm

    let mut freq_sweep = Vec::new();
    for i in 0..num_points {
        let freq_ghz = (i as f64 + 1.0) / num_points as f64 * max_freq_ghz;
        let omega = 2.0 * std::f64::consts::PI * freq_ghz * 1e9; // rad/s
        
        // Frequency-dependent impedance (simplified)
        let freq_impedance = z0 * (1.0 + 0.01 * freq_ghz.sqrt()); // Skin effect increases Z
        let phase = omega * delay * 1e-12 * 180.0 / std::f64::consts::PI; // degrees
        let attenuation = alpha * trace_len * freq_ghz.sqrt(); // dB
        
        freq_sweep.push(FrequencyPoint {
            frequency_ghz: freq_ghz,
            impedance_ohms: freq_impedance,
            phase_deg: phase % 360.0,
            attenuation_db: attenuation,
        });
    }

    TransmissionLineResult {
        characteristic_impedance_ohms: z0,
        propagation_constant: alpha + beta * max_freq_ghz,
        phase_velocity_mm_ps: vp,
        group_velocity_mm_ps: vp * (1.0 - 0.01 * max_freq_ghz.sqrt()), // Simplified dispersion
        wavelength_mm: vp / (max_freq_ghz * 1e3), // mm at max freq
        attenuation_db_per_m: trace.attenuation_db_per_m,
        frequency_sweep: freq_sweep,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::pcb::*;
    use crate::model::id::*;

    #[test]
    fn test_trace_length_simple() {
        let trace = Trace {
            net_id: None, layer: LayerId(1), width: 0.25,
            points: vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0)],
            attenuation_db_per_m: 0.0,
        };
        assert!((trace_length(&trace) - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_si_analysis_runs() {
        let mut board = PcbBoard::default();
        board.traces.insert(Trace {
            net_id: None, layer: LayerId(1), width: 0.25,
            points: vec![Point::new(0.0, 0.0), Point::new(50.0, 0.0)],
            attenuation_db_per_m: 0.0,
        });
        let results = analyze_board_si(&board);
        assert_eq!(results.len(), 1);
        assert!(results[0].trace_length_mm > 0.0);
    }
}
