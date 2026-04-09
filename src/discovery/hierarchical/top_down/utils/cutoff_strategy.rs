pub trait CutoffStrategy {
    fn cutoff(&self, values: &[u32]) -> u32;
}

pub struct HighestDifference;

impl CutoffStrategy for HighestDifference {
    fn cutoff(&self, values: &[u32]) -> u32 {
        let mut sorted = values.to_vec();
        sorted.sort_unstable_by(|a, b| b.cmp(a));

        sorted.windows(2)
            .max_by_key(|w| w[0] - w[1])
            .map(|w| w[0])
            .unwrap_or(sorted[0])
    }
}
