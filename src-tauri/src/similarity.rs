use std::collections::HashSet;

pub fn token_overlap(left: &str, right: &str) -> f64 {
    let left_tokens = tokenize(left);
    let right_tokens = tokenize(right);
    if left_tokens.is_empty() || right_tokens.is_empty() {
        return 0.0;
    }

    let intersection = left_tokens.intersection(&right_tokens).count() as f64;
    let union = left_tokens.union(&right_tokens).count() as f64;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

fn tokenize(input: &str) -> HashSet<String> {
    input
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| part.chars().count() >= 2)
        .map(|part| part.to_lowercase())
        .collect()
}

pub fn has_version_noise(name: &str) -> bool {
    let lower = name.to_lowercase();
    [
        "v1", "v2", "v3", "new", "final", "backup", "copy", "old", "旧版", "备份", "最终",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}
