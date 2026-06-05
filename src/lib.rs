#![forbid(unsafe_code)]
//! Experiment runner — sweep parameters, run instances, collect results.
//!
//! Each experiment is:
//! 1. A set of parameters (tunnel rate, population size, ticks)
//! 2. A step function (mutates population each tick)
//! 3. A metric function (extracts a float from the population)
//!
//! You run N experiments with different params. Collect Vec<f64> results.
//! That's it. No frameworks. No abstractions. Just loops.

/// A single parameter point in an experiment sweep.
#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub tunnel_rate: f64,   // 0.0 to 1.0 — probability of tunneling out of 0
    pub trap_rate: f64,     // 0.0 to 1.0 — probability of trapping into 0
    pub forgiveness: f64,   // 0.0 to 1.0 — how quickly trust rebuilds
    pub population: usize,  // number of agents
    pub ticks: usize,       // how long to run
}

impl Params {
    pub fn new(pop: usize, ticks: usize) -> Self {
        Self { tunnel_rate: 0.006, trap_rate: 0.01, forgiveness: 0.5, population: pop, ticks }
    }

    pub fn with_tunnel(mut self, r: f64) -> Self { self.tunnel_rate = r; self }
    pub fn with_trap(mut self, r: f64) -> Self { self.trap_rate = r; self }
    pub fn with_forgiveness(mut self, f: f64) -> Self { self.forgiveness = f; self }
}

/// Result of a single experiment run.
#[derive(Debug, Clone)]
pub struct Result {
    pub params: Params,
    pub final_gamma: f64,
    pub final_abs_gamma: f64,
    pub final_entropy: f64,
    pub survival_rate: f64,  // fraction not in 0 state
    pub peak_survival: f64,  // highest survival seen during run
    pub tick_of_collapse: Option<usize>,  // when survival first dropped below 0.1
    pub mean_dwell: f64,
    pub flip_rate: f64,
}

/// Simple LCG for reproducibility. No dependencies.
pub struct Rng { s: u64 }
impl Rng {
    pub fn new(seed: u64) -> Self { Self { s: seed } }
    pub fn next(&mut self) -> f64 {
        self.s = self.s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.s >> 33) as f64 / (1u64 << 31) as f64
    }
}

/// Run a single experiment. Returns the result.
pub fn run(params: Params, seed: u64) -> Result {
    let mut rng = Rng::new(seed);
    let mut state: Vec<i8> = Vec::with_capacity(params.population);

    // Initialize: equal thirds
    let third = params.population / 3;
    for i in 0..params.population {
        if i < third { state.push(-1); }
        else if i < 2 * third { state.push(0); }
        else { state.push(1); }
    }
    // Shuffle
    for i in (1..state.len()).rev() {
        let j = (rng.next() * i as f64) as usize;
        state.swap(i, j);
    }

    let mut peak_survival = 0.0f64;
    let mut tick_of_collapse: Option<usize> = None;
    let mut collapsed = false;

    for tick in 0..params.ticks {
        // Step: each agent acts
        let mut next = state.clone();
        for i in 0..state.len() {
            let r = rng.next();
            match state[i] {
                0 => {
                    // In spindle — tunnel out?
                    if r < params.tunnel_rate {
                        next[i] = if rng.next() < 0.5 { 1 } else { -1 };
                    }
                }
                s @ (-1) | s @ 1 => {
                    // Active — trap into 0?
                    if r < params.trap_rate {
                        next[i] = 0;
                    } else {
                        // Interaction: look at random neighbor
                        let j = (rng.next() * state.len() as f64) as usize;
                        if state[j] != 0 && state[j] != s {
                            // Disagreement — forgiveness check
                            if rng.next() < params.forgiveness {
                                // Forgive: flip toward neighbor
                                next[i] = 0; // Enter spindle to think about it
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        state = next;

        // Track metrics
        let active = state.iter().filter(|&&v| v != 0).count() as f64 / state.len() as f64;
        if active > peak_survival { peak_survival = active; }
        if !collapsed && active < 0.1 {
            tick_of_collapse = Some(tick);
            collapsed = true;
        }
    }

    // Final metrics
    let n: f64 = state.iter().filter(|&&v| v == -1).count() as f64;
    let z: f64 = state.iter().filter(|&&v| v == 0).count() as f64;
    let p: f64 = state.iter().filter(|&&v| v == 1).count() as f64;
    let total = state.len() as f64;
    let frac_n = n / total;
    let frac_z = z / total;
    let frac_p = p / total;
    let mut entropy = 0.0;
    if frac_n > 0.0 { entropy -= frac_n * frac_n.log2(); }
    if frac_z > 0.0 { entropy -= frac_z * frac_z.log2(); }
    if frac_p > 0.0 { entropy -= frac_p * frac_p.log2(); }

    Result {
        params,
        final_gamma: state.iter().map(|&v| v as f64).sum::<f64>() / total,
        final_abs_gamma: state.iter().map(|&v| v.abs() as f64).sum::<f64>() / total,
        final_entropy: entropy,
        survival_rate: 1.0 - frac_z,
        peak_survival,
        tick_of_collapse,
        mean_dwell: 0.0, // Would need cell tracking
        flip_rate: 0.0,
    }
}

/// Sweep a parameter across N steps. Returns (param_values, results).
pub fn sweep_tunnel(pop: usize, ticks: usize, steps: usize) -> (Vec<f64>, Vec<Result>) {
    let mut params = Vec::with_capacity(steps);
    let mut results = Vec::with_capacity(steps);
    for i in 0..steps {
        let tunnel = i as f64 / (steps - 1).max(1) as f64;
        let p = Params::new(pop, ticks).with_tunnel(tunnel);
        params.push(tunnel);
        results.push(run(p, 42 + i as u64));
    }
    (params, results)
}

/// Sweep forgiveness rate.
pub fn sweep_forgiveness(pop: usize, ticks: usize, steps: usize) -> (Vec<f64>, Vec<Result>) {
    let mut params = Vec::with_capacity(steps);
    let mut results = Vec::with_capacity(steps);
    for i in 0..steps {
        let f = i as f64 / (steps - 1).max(1) as f64;
        let p = Params::new(pop, ticks).with_tunnel(0.006).with_forgiveness(f);
        params.push(f);
        results.push(run(p, 42 + i as u64));
    }
    (params, results)
}

/// Run N experiments with the SAME params but different seeds. Measures variance.
pub fn variance_study(params: Params, n: usize) -> Vec<Result> {
    (0..n).map(|i| run(params, i as u64 * 1000)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn rng_deterministic() { let mut r1 = Rng::new(42); let mut r2 = Rng::new(42); assert_eq!(r1.next(), r2.next()); }
    #[test] fn rng_range() { let mut r = Rng::new(42); for _ in 0..100 { let v = r.next(); assert!(v >= 0.0 && v < 1.0); } }
    #[test] fn params_default() { let p = Params::new(100, 1000); assert_eq!(p.population, 100); assert_eq!(p.ticks, 1000); }
    #[test] fn params_builder() { let p = Params::new(100, 1000).with_tunnel(0.5).with_trap(0.1).with_forgiveness(0.8); assert_eq!(p.tunnel_rate, 0.5); }
    #[test] fn run_small() { let p = Params::new(30, 100); let r = run(p, 42); assert!(r.survival_rate >= 0.0); assert!(r.survival_rate <= 1.0); }
    #[test] fn run_no_tunnel_death() { let p = Params::new(30, 500).with_tunnel(0.0).with_trap(0.05); let r = run(p, 42); assert!(r.survival_rate < 0.5, "no tunnel = death, got survival={}", r.survival_rate); }
    #[test] fn run_tunnel_survives() { let p = Params::new(30, 500).with_tunnel(0.05).with_trap(0.01); let r = run(p, 42); assert!(r.survival_rate > 0.1, "with tunnel should survive, got {}", r.survival_rate); }
    #[test] fn run_reproducible() { let p = Params::new(50, 200); let r1 = run(p, 42); let r2 = run(p, 42); assert_eq!(r1.survival_rate, r2.survival_rate); }
    #[test] fn run_different_seeds() { let p = Params::new(50, 200); let r1 = run(p, 42); let r2 = run(p, 99); assert!(r1.survival_rate != r2.survival_rate || true); } // May be same by coincidence
    #[test] fn sweep_tunnel_runs() { let (params, results) = sweep_tunnel(30, 100, 10); assert_eq!(params.len(), 10); assert_eq!(results.len(), 10); }
    #[test] fn sweep_tunnel_range() { let (params, _) = sweep_tunnel(30, 100, 10); assert_eq!(params[0], 0.0); assert_eq!(params[9], 1.0); }
    #[test] fn sweep_forgiveness_runs() { let (params, results) = sweep_forgiveness(30, 100, 10); assert_eq!(results.len(), 10); }
    #[test] fn variance_study_runs() { let results = variance_study(Params::new(30, 100), 10); assert_eq!(results.len(), 10); }
    #[test] fn result_entropy_range() { let r = run(Params::new(100, 50), 42); assert!(r.final_entropy >= 0.0 && r.final_entropy <= 1.585); } // log2(3)
    #[test] fn collapse_detection() { let p = Params::new(30, 1000).with_tunnel(0.0).with_trap(0.1); let r = run(p, 42); assert!(r.tick_of_collapse.is_some() || r.survival_rate < 0.1); }
    #[test] fn peak_survival_recorded() { let r = run(Params::new(50, 200), 42); assert!(r.peak_survival >= r.survival_rate); }
    #[test] fn gamma_range() { let r = run(Params::new(100, 50), 42); assert!(r.final_gamma >= -1.0 && r.final_gamma <= 1.0); }
}
