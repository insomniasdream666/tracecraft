use crate::model::pcb::{LayerStack, ImpedanceResult};

const COPPER_RESISTIVITY: f64 = 1.68e-8; // ohm·m

/// Calculate microstrip trace impedance and required width for a target impedance.
/// Uses the IPC-2141A microstrip formula.
pub fn microstrip_impedance(
    target_impedance: f64,
    stackup: &LayerStack,
    layer_index: usize,
    trace_thickness_mm: f64,
) -> ImpedanceResult {
    let (er, height_mm) = find_dielectric_below(stackup, layer_index);

    // IPC-2141A microstrip: Z0 = 87 / sqrt(Er + 1.41) * ln(5.98*H / (0.8*W + T))
    // Solve for W given Z0
    let h = height_mm.max(0.01);
    let t = trace_thickness_mm.max(0.001);
    let coef = 87.0 / (er + 1.41).sqrt();
    let arg = (target_impedance / coef).exp();
    // arg = 5.98 * H / (0.8*W + T) => 0.8*W + T = 5.98*H / arg => W = ((5.98*H / arg) - T) / 0.8
    let w = ((5.98 * h / arg) - t) / 0.8;
    let width_mm = w.max(0.05);

    // Recalculate actual impedance from the computed width
    let actual_z = coef * (5.98 * h / (0.8 * width_mm + t)).ln();

    ImpedanceResult {
        trace_width_mm: width_mm,
        impedance_ohms: actual_z,
        trace_thickness_mm: t,
    }
}

/// Calculate differential microstrip impedance for edge-coupled microstrip.
pub fn differential_impedance(
    single_ended_z: f64,
    gap_mm: f64,
    height_mm: f64,
    er: f64,
) -> f64 {
    // Zdiff ≈ 2 * Z0 * (1 - 0.48 * exp(-0.96 * S / H))
    let s_over_h = (gap_mm / height_mm.max(0.01)).max(0.01);
    let coupling = 1.0 - 0.48 * (-0.96 * s_over_h).exp();
    single_ended_z * 2.0 * coupling
}

fn find_dielectric_below(stackup: &LayerStack, layer_index: usize) -> (f64, f64) {
    if let Some(diel) = stackup.dielectrics.first() {
        (diel.er, diel.thickness_mm)
    } else {
        (4.5, 1.6)
    }
}

/// Calculate copper thickness for given weight
pub fn copper_thickness_mm(oz: f64) -> f64 {
    // 1 oz = 0.035 mm
    oz * 0.035
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::pcb::LayerStack;

    #[test]
    fn test_microstrip_50_ohm() {
        let stackup = LayerStack::default_2layer();
        let t = copper_thickness_mm(1.0);
        let result = microstrip_impedance(50.0, &stackup, 0, t);
        // Should give approximately 50 ohms
        assert!((result.impedance_ohms - 50.0).abs() < 5.0);
        assert!(result.trace_width_mm > 0.1);
    }

    #[test]
    fn test_copper_thickness() {
        let t = copper_thickness_mm(1.0);
        assert!((t - 0.035).abs() < 0.001);
    }
}
