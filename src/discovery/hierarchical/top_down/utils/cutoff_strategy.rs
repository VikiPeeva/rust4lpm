pub trait CutoffStrategy {
    fn cutoff(&self, values: &[u32]) -> u32;
}

pub struct HighestDifference;

impl CutoffStrategy for HighestDifference {
    fn cutoff(&self, values: &[u32]) -> u32 {
        todo!()
    }
}
