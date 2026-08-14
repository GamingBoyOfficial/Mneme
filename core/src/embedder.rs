pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> Vec<f32>;
}

/// Deterministic embedder using character trigrams hashing.
/// Not semantically meaningful, but serves as a placeholder for testing.
pub struct HashEmbedder {
    dim: usize,
}

impl HashEmbedder {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl Embedder for HashEmbedder {
    fn embed(&self, text: &str) -> Vec<f32> {
        let mut vec = vec![0.0; self.dim];
        let bytes = text.as_bytes();
        for i in 0..bytes.len().saturating_sub(2) {
            let trigram = &bytes[i..i+3];
            let hash = (trigram[0] as usize * 31 + trigram[1] as usize * 7 + trigram[2] as usize) % self.dim;
            vec[hash] += 1.0;
        }
        // Normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut vec { *x /= norm; }
        }
        vec
    }
}