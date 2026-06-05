# ternary-experiment

**Parameter sweep runner for ternary agent simulations. Run 10,000 worlds.**

You have a hypothesis: *"If the tunnel rate is above 0.3%, the population survives."* How do you test it? You run a thousand simulations with different parameters, measure the outcomes, and look for the pattern. This crate is the scaffolding for that: define parameters, run experiments, collect results.

No frameworks. No abstractions. Just loops with metrics.

## What's Inside

- **`Params`** — parameter point: `tunnel_rate`, `trap_rate`, `forgiveness`, `population`, `ticks`
- **`Result`** — outcome: `final_gamma`, `survival_rate`, `peak_survival`, `tick_of_collapse`, `entropy`, `flip_rate`
- **`Rng`** — deterministic LCG for reproducibility. Same seed = same result, every time
- **`run(params, seed)`** — single experiment. Initialize equal-thirds population, step, measure
- **`sweep(param_ranges, seeds)`** — batch experiments across parameter space
- **`SweepResult`** — aggregated results with mean, variance, best/worst case

## Quick Example

```rust
use ternary_experiment::*;

// One experiment: 100 agents, 1000 ticks, moderate parameters
let params = Params::new(100, 1000)
    .with_tunnel(0.006)
    .with_trap(0.01)
    .with_forgiveness(0.5);

let result = run(params, 42);
println!("Survival: {:.1}%", result.survival_rate * 100.0);
println!("Collapsed at tick: {:?}", result.tick_of_collapse);

// Sweep: vary tunnel rate, keep everything else constant
let mut results = Vec::new();
for i in 0..100 {
    let tunnel = 0.001 + (i as f64) * 0.001; // 0.001 to 0.100
    let p = Params::new(100, 1000).with_tunnel(tunnel);
    results.push(run(p, i as u64));
}

// Find the critical tunnel rate
let critical = results.iter()
    .filter(|r| r.survival_rate > 0.5)
    .map(|r| r.params.tunnel_rate)
    .reduce(f64::min);
println!("Minimum tunnel rate for survival: {:?}", critical);
```

## The Insight

**Phase transitions have thresholds.** In ternary agent systems, there's a critical tunnel rate below which the population collapses to 0 (the monoculture trap). This crate helps you *find* that threshold — not analytically, but empirically, by sweeping parameter space. The `Rng` is deterministic, so results are reproducible. The metrics are quantitative, so you can plot survival curves.

**Use cases:**
- **Agent simulation research** — sweep parameters, find phase transitions
- **Systems tuning** — find the critical parameters for your agent fleet
- **Teaching** — demonstrate emergent behavior through parameter exploration
- **Benchmarking** — reproducible experiments with deterministic seeds
- **Monte Carlo methods** — run N simulations, aggregate statistics

## See Also

- **ternary-drift** — the genetic drift dynamics inside each experiment
- **ternary-grace** — grace vs. trust parameters that experiments sweep over
- **ternary-cell** — million-instance scale for large experiments
- **ternary-entropy** — entropy metrics computed during experiments
- **ternary-popgen** — population genetics view of experiment results

## Install

```bash
cargo add ternary-experiment
```

## License

MIT
