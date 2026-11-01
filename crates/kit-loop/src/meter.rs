//! Tokens and dollars (PRD §5.6). Prices are per million tokens. A model with
//! no price shows tokens only: Kit never guesses a price.

use kit_providers::Usage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One model's prices, in USD per million tokens.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct Price {
    /// Uncached input.
    pub input: f64,
    /// Input read from the cache.
    #[serde(default)]
    pub cached_input: Option<f64>,
    /// Input written to the cache.
    #[serde(default)]
    pub cache_write: Option<f64>,
    /// Output, reasoning included.
    pub output: f64,
}

/// Prices by model id, plus the date they were read.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Prices {
    /// When these prices were checked, e.g. `2026-10-31`.
    #[serde(default)]
    pub as_of: String,
    /// Model id → price.
    #[serde(default)]
    pub models: BTreeMap<String, Price>,
}

impl Prices {
    /// Parse a `prices.toml`.
    pub fn from_toml(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| format!("prices.toml: {e}"))
    }

    /// Cost of one reply, if the model has a price.
    pub fn cost(&self, model: &str, u: &Usage) -> Option<f64> {
        let p = self.models.get(model)?;
        let m = 1_000_000.0;
        Some(
            u.input as f64 * p.input / m
                + u.cached_input as f64 * p.cached_input.unwrap_or(p.input) / m
                + u.cache_write as f64 * p.cache_write.unwrap_or(p.input) / m
                + u.output as f64 * p.output / m,
        )
    }
}

/// Running totals for a session.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Meter {
    /// Summed token counts.
    pub tokens: Usage,
    /// Dollars, when every reply had a cost.
    pub cost_usd: Option<f64>,
    /// True when any cost was computed from a price table, not reported.
    pub estimated: bool,
    /// Replies so far.
    pub replies: u32,
}

impl Meter {
    /// Add one reply. A reply with no known cost makes the total unknown.
    pub fn add(&mut self, u: &Usage, computed: Option<f64>) {
        let t = &mut self.tokens;
        t.input += u.input;
        t.cached_input += u.cached_input;
        t.cache_write += u.cache_write;
        t.output += u.output;
        t.reasoning += u.reasoning;
        let reply = u.cost_usd.or(computed);
        if u.cost_usd.is_none() && computed.is_some() {
            self.estimated = true;
        }
        self.cost_usd = match (self.replies, self.cost_usd, reply) {
            (0, _, r) => r,
            (_, Some(total), Some(r)) => Some(total + r),
            _ => None,
        };
        self.replies += 1;
    }

    /// `≈$0.041`, `$0.041` or `$?`.
    pub fn dollars(&self) -> String {
        match self.cost_usd {
            Some(c) => format!("{}${c:.3}", if self.estimated { "≈" } else { "" }),
            None => "$?".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_from_prices_and_marks_estimates() {
        let prices = Prices::from_toml(
            "as_of = \"2026-10-31\"\n[models.\"m\"]\ninput = 3.0\ncached_input = 0.3\noutput = 15.0\n",
        )
        .unwrap();
        let u = Usage {
            input: 1_000_000,
            cached_input: 1_000_000,
            output: 100_000,
            ..Default::default()
        };
        let c = prices.cost("m", &u).unwrap();
        assert!((c - 4.8).abs() < 1e-9, "{c}");
        let mut m = Meter::default();
        m.add(&u, Some(c));
        assert_eq!(m.dollars(), "≈$4.800");
        assert_eq!(prices.cost("unknown", &u), None, "no price, no guess");
    }

    #[test]
    fn reported_cost_wins_and_unknown_makes_the_total_unknown() {
        let mut m = Meter::default();
        m.add(
            &Usage {
                cost_usd: Some(0.01),
                ..Default::default()
            },
            Some(99.0),
        );
        assert_eq!(m.dollars(), "$0.010");
        m.add(&Usage::default(), None);
        assert_eq!(m.dollars(), "$?");
        assert_eq!(m.replies, 2);
    }
}
