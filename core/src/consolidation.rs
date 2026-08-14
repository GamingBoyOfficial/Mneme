use crate::store::MemoryRecord;

/// Returns IDs of duplicate memories that should be removed.
/// Keeps the one with highest confidence (or latest timestamp if confidence equal).
pub fn find_duplicates(records: &[MemoryRecord], threshold: f32) -> Vec<String> {
    let mut to_remove = Vec::new();
    for i in 0..records.len() {
        if to_remove.contains(&records[i].id) {
            continue;
        }
        for j in (i + 1)..records.len() {
            if to_remove.contains(&records[j].id) {
                continue;
            }
            let sim = jaccard_similarity(&records[i].content, &records[j].content);
            if sim > threshold {
                // Keep the one with higher confidence; if equal, keep earlier timestamp
                if records[i].confidence >= records[j].confidence {
                    to_remove.push(records[j].id.clone());
                } else {
                    to_remove.push(records[i].id.clone());
                }
            }
        }
    }
    to_remove
}

fn jaccard_similarity(a: &str, b: &str) -> f32 {
    let set_a: std::collections::HashSet<_> = a.split_whitespace().collect();
    let set_b: std::collections::HashSet<_> = b.split_whitespace().collect();
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 { 0.0 } else { intersection as f32 / union as f32 }
}