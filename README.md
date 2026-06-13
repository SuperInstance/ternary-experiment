# Ternary Experiment — Lightweight Parameter Sweeps for Ternary Agent Simulations

**Ternary Experiment** is a zero-dependency experiment runner for ternary agent simulations. It sweeps parameters (tunnel rate, trap rate, forgiveness, population size, ticks), runs stochastic simulations with seeded RNGs, and collects metrics (γ, entropy, survival rate, dwell time, flip rate) — all without external frameworks or abstractions.

## Why It Matters

Scientific progress requires reproducible experiments. This crate provides the minimal scaffolding: define parameters, run simulations, collect floats. No YAML configs, no framework lock-in, no distributed compute orchestration — just `run(params, seed) → Result`. The seeded RNG ensures that any experiment can be reproduced exactly, and the built-in LCG (Linear Congruential Generator) has no external dependencies. For ternary research, this is the workhorse that generates the data backing the γ + η = C conservation law.

## How It Works

### Parameter Model

`Params` captures the five knobs that govern ternary agent dynamics:

- `tunnel_rate` (0.0-1.0): probability of escaping the 0 state
- `trap_rate` (0.0-1.0): probability of falling into the 0 state
- `forgiveness` (0.0-1.0): how quickly trust rebuilds after conflict
- `population`: number of agents
- `ticks`: simulation duration

### Simulation Loop

Each tick:
1. For each agent, check if it tunnels out of 0 (with probability `tunnel_rate`)
2. Check if it traps into 0 (with probability `trap_rate`)
3. Apply forgiveness to rebuild trust
4. Record metrics

The simulation maintains a population vector `Vec<i8>` of agent states and produces time-series metrics.

### Metrics

The `Result` struct captures:

- **final_gamma**: mean state value (positive = growth-dominated)
- **final_abs_gamma**: mean absolute state value (measures polarization)
- **final_entropy**: Shannon entropy of state distribution (0 = uniform)
- **survival_rate**: fraction of agents not in state 0
- **peak_survival**: highest survival during the run
- **tick_of_collapse**: when survival first drops below 0.1
- **mean_dwell**: average ticks per state before transitioning
- **flip_rate**: fraction of agents that changed state

### Reproducibility

The `Rng` is a simple LCG: `s = s · 6364136223846793005 + 1442695040888963407`, extracting bits 33-63 as a float in [0, 1). Same seed → same sequence. No system entropy, no thread-local state.

## Quick Start

```rust
use ternary_experiment::{Params, run};

// Run a baseline experiment
let params = Params::new(300, 1000);
let result = run(params, 42); // seed = 42
println!("Survival rate: {:.1}%", result.survival_rate * 100.0);

// Sweep tunnel rate
for tunnel in [0.001, 0.005, 0.01, 0.05] {
    let p = Params::new(300, 1000).with_tunnel(tunnel);
    let r = run(p, 42);
    println!("tunnel={}: survival={:.1}%, entropy={:.3}", tunnel, r.survival_rate * 100.0, r.final_entropy);
}
```

```bash
cargo add ternary-experiment
```

## API

| Type / Function | Description |
|---|---|
| `Params` | `new(pop, ticks)`, `.with_tunnel()`, `.with_trap()`, `.with_forgiveness()` |
| `run(Params, seed)` | Run one experiment → `Result` |
| `Result` | Final metrics: gamma, entropy, survival, collapse tick |
| `Rng` | Seeded LCG: `new(seed)`, `next() → f64` |

## Architecture Notes

This is the experimental backbone of **SuperInstance** research. Every claim about the γ + η = C conservation law is backed by experiments run through this crate. The parameters directly model the conservation dynamics: `tunnel_rate` controls γ (escape from entropy), `trap_rate` controls η (fall into entropy), and their balance determines whether the system sustains diversity. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Axelrod, Robert. *The Complexity of Cooperation*, Princeton UP, 1997 — agent-based simulation methodology.
- Nowak, Martin. *Evolutionary Dynamics*, Harvard UP, 2006 — stochastic game dynamics.
- Press, William H. et al. *Numerical Recipes*, 3rd ed., Cambridge UP, 2007 — LCG quality and seeding.

## License

MIT
