pub(super) const MAX_PLAN_CHARS: usize = 80_000;
pub(super) const CHUNK_SIZE_CHARS: usize = 20_000;

pub(super) fn chunk_large_plan(plan: &str) -> Vec<String> {
    if plan.len() <= MAX_PLAN_CHARS {
        return vec![plan.to_string()];
    }
    plan.chars()
        .collect::<Vec<char>>()
        .chunks(CHUNK_SIZE_CHARS)
        .map(|chars| chars.iter().collect())
        .collect()
}
