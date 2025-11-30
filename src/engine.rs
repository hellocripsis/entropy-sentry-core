use rand::rngs::OsRng;
use rand::RngCore;

use crate::metrics::EntropyMetrics;
use crate::sentry::SentryDecision;

/// Configuration for the entropy engine + sentry decision.
#[derive(Debug, Clone, Copy)]
pub struct EntropyConfig {
    /// Minimum samples before we trust the metrics at all.
    pub min_samples: u64,
    /// Expected center for bit-density (p), usually 0.5.
    pub mean_center: f64,
    /// Allowed deviation around mean_center before we worry.
    pub mean_tolerance: f64,
    /// Jitter level where we start throttling.
    pub max_jitter: f64,
    /// Jitter level where we kill.
    pub kill_jitter: f64,
}

impl Default for EntropyConfig {
    fn default() -> Self {
        Self {
            min_samples: 256,
            mean_center: 0.5,
            mean_tolerance: 0.05, // +/- 0.05 around 0.5
            max_jitter: 0.10,
            kill_jitter: 0.18,
        }
    }
}

impl EntropyConfig {
    /// More sensitive profile – easier to trigger Throttle/Kill.
    pub fn aggressive() -> Self {
        Self {
            min_samples: 128,
            mean_center: 0.5,
            mean_tolerance: 0.03,
            max_jitter: 0.07,
            kill_jitter: 0.14,
        }
    }
}

/// Entropy engine: samples OsRng, tracks metrics, and produces a decision.
#[derive(Debug)]
pub struct EntropyEngine {
    cfg: EntropyConfig,
    rng: OsRng,
    samples: Vec<f64>,
}

impl EntropyEngine {
    /// Create engine with the given config.
    pub fn with_config(cfg: EntropyConfig) -> Self {
        Self {
            cfg,
            rng: OsRng,
            samples: Vec::new(),
        }
    }

    /// Create engine with default config.
    pub fn new() -> Self {
        Self::with_config(EntropyConfig::default())
    }

    /// Take one sample from OsRng, return `p` (bit-density in [0,1]).
    pub fn sample(&mut self) -> f64 {
        let x: u64 = self.rng.next_u64();
        let bits_set = x.count_ones() as f64;
        let p = bits_set / 64.0;
        self.samples.push(p);
        p
    }

    /// Compute current metrics over all samples seen so far.
    pub fn metrics(&self) -> EntropyMetrics {
        EntropyMetrics::from_samples(&self.samples)
    }

    /// Compute a sentry decision from current metrics.
    pub fn decision(&self) -> SentryDecision {
        let m = self.metrics();

        // Not enough data yet – do nothing dramatic.
        if (m.sample_count as u64) < self.cfg.min_samples {
            return SentryDecision::Keep;
        }

        let mean_delta = (m.mean - self.cfg.mean_center).abs();

        // Hard fail conditions.
        if mean_delta > self.cfg.mean_tolerance || m.jitter > self.cfg.kill_jitter {
            return SentryDecision::Kill;
        }

        // Softer anomaly conditions.
        if mean_delta > self.cfg.mean_tolerance * 0.5 || m.jitter > self.cfg.max_jitter {
            return SentryDecision::Throttle;
        }

        SentryDecision::Keep
    }
}
