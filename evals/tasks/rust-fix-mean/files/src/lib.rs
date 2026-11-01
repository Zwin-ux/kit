pub fn mean(xs: &[f64]) -> Option<f64> {
    Some(xs.iter().sum::<f64>() / (xs.len() - 1) as f64)
}
