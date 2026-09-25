//! What a run of the harness measured, and the baselines beside it.
//!
//! Every accuracy here is printed next to the score a constant answer would
//! get, because AT01 measured an intent head at 0.697 where always answering
//! `FindResource` scores 0.737 -- and without the baseline in the same table
//! that reads as a result. A number whose baseline is absent is not reported.

use std::collections::BTreeMap;

/// The commonest intent's share: what a constant answer scores.
pub fn majority(intents: &[String]) -> f64 {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for intent in intents {
        *counts.entry(intent.as_str()).or_default() += 1;
    }
    let best = counts.values().copied().max().unwrap_or_default();
    if intents.is_empty() {
        return 0.0;
    }
    best as f64 / intents.len() as f64
}

/// McNemar's exact-ish test on two arms over the same questions.
///
/// Returns the discordant counts and a two-sided p-value from the binomial
/// with p = 0.5. Paired, because the arms answered the same questions: what
/// matters is how often one is right where the other is wrong, not how often
/// they agree.
pub fn mcnemar(a: &[bool], b: &[bool]) -> (usize, usize, f64) {
    let pairs = a.iter().zip(b);
    let wins = pairs.clone().filter(|(x, y)| **x && !**y).count();
    let losses = pairs.filter(|(x, y)| !**x && **y).count();
    let n = wins + losses;
    if n == 0 {
        return (0, 0, 1.0);
    }
    let lower = wins.min(losses);
    let mut tail = 0.0;
    for k in 0..=lower {
        tail += binomial(n, k);
    }
    (wins, losses, (2.0 * tail).min(1.0))
}

/// The binomial probability of exactly `k` of `n` at p = 0.5, computed in
/// logarithms so 400 questions do not overflow.
fn binomial(n: usize, k: usize) -> f64 {
    let ln_factorial = |m: usize| (1..=m).map(|i| (i as f64).ln()).sum::<f64>();
    let ln_choose = ln_factorial(n) - ln_factorial(k) - ln_factorial(n - k);
    (ln_choose - (n as f64) * 2.0_f64.ln()).exp()
}

/// A 95% interval for the difference between two arms, by paired bootstrap.
///
/// Deterministic: the resampling uses a fixed seed, so the interval a reader
/// sees is the interval they get when they rerun it.
pub fn interval(a: &[bool], b: &[bool], rounds: usize) -> (f64, f64) {
    let n = a.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    let mut seed = 0x5EED_1234_u64;
    let mut margins: Vec<f64> = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let mut sum = 0.0;
        for _ in 0..n {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let pick = (seed >> 33) as usize % n;
            sum += f64::from(u8::from(a[pick])) - f64::from(u8::from(b[pick]));
        }
        margins.push(sum / n as f64);
    }
    margins.sort_by(f64::total_cmp);
    (margins[rounds / 40], margins[rounds - rounds / 40 - 1])
}
