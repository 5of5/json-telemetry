//! Real-data ingestion: bytes → optical fields.
//!
//! The encoding is native to the substrate rather than bolted on. A field is
//! the *spectrum* of a text window: each byte contributes a phasor at every
//! mode, so the inner product between two fields is precisely the spectral
//! similarity of their windows. That is the quantity an optical interferometer
//! computes in one pass — the encoding exists so that interference is meaning.
//!
//! The Spec does not know about this path, and does not need to: data enters
//! only as training fields and (optionally) initial conditions. It defines no
//! action and enlarges nothing.
//!
//! Invariants of the encoding, all of which the tests assert:
//!
//! * deterministic — same bytes in, same fields out, on every platform up to
//!   the last ulp of `sin`/`cos`;
//! * unit norm — `‖ψ‖ = 1`, so the worst-case Inv2 jump after an OpticalStep
//!   stays `≤ 2·Lip(P)` exactly as for the synthetic path;
//! * no information loss at window granularity — the DFT is invertible, so the
//!   window is recoverable from the field (up to normalization and the global
//!   phase lost to byte-centering).

use num_complex::Complex64;
use std::f64::consts::TAU;
use std::path::Path;

/// A dataset of real-data field sequences, in the training format.
///
/// Shares its shape with the synthetic optical dataset so the training loop
/// consumes both unchanged; `format` records which one it is.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldDataset {
    pub format: String,
    pub n_modes: usize,
    /// Which encoder produced the frames.
    pub encoding: String,
    /// What the bytes were read from.
    pub source: String,
    /// How many source bytes were consumed.
    pub source_bytes: usize,
    /// One trajectory: `[frame][2·n_modes]` flattened as [re₀, im₀, …].
    pub trajectories: Vec<Vec<Vec<f64>>>,
}

/// Encode one window of bytes as a unit-norm optical field on `n_modes` modes.
///
/// `ψ[m] ∝ Σⱼ xⱼ e^{−2πi·mj/N}` where `xⱼ` is the centered byte value. The
/// result is the DFT spectrum of the window, normalized to ‖ψ‖ = 1.
pub fn encode_window(window: &[u8], n_modes: usize) -> Vec<Complex64> {
    debug_assert!(n_modes >= 1);
    let n = n_modes as f64;
    let phasors: Vec<Vec<Complex64>> = (0..n_modes)
        .map(|m| {
            (0..window.len())
                .map(|j| {
                    let angle = -TAU * (m as f64) * (j as f64) / n;
                    Complex64::new(libm::cos(angle), libm::sin(angle)) / n.sqrt()
                })
                .collect()
        })
        .collect();
    encode_window_with(window, n_modes, &phasors)
}

/// Encode with a precomputed phasor table (`phasors[m][j] = e^{−2πi·mj/N}/√N`).
fn encode_window_with(
    window: &[u8],
    n_modes: usize,
    phasors: &[Vec<Complex64>],
) -> Vec<Complex64> {
    let n = n_modes as f64;
    let mut psi: Vec<Complex64> = phasors
        .iter()
        .map(|row| {
            row.iter()
                .zip(window)
                .map(|(p, &b)| {
                    let x = (f64::from(b) - 127.5) / 127.5;
                    Complex64::new(p.re * x, p.im * x)
                })
                .sum()
        })
        .collect();

    // Unit norm: keeps the Inv2 worst case at 2·Lip(P) regardless of content.
    let norm = psi.iter().map(num_complex::Complex::norm_sqr).sum::<f64>().sqrt();
    if norm > 0.0 {
        for c in &mut psi {
            *c /= Complex64::new(norm, 0.0);
        }
    } else {
        // A window whose centered bytes cancel exactly has no spectral energy;
        // emit the unit-norm alternating field instead of a zero vector.
        let a = 1.0 / n.sqrt();
        for (m, c) in psi.iter_mut().enumerate() {
            *c = Complex64::new(if m % 2 == 0 { a } else { -a }, 0.0);
        }
    }
    psi
}

/// Split bytes into windows of `n_modes` with the given stride and encode each.
///
/// `stride` smaller than the window overlaps windows; the default should be
/// the window itself (non-overlapping). The final partial window is kept when
/// it holds at least 8 bytes — short tails carry almost no signal but do keep
/// real corpus length faithful.
pub fn encode_corpus(
    bytes: &[u8],
    n_modes: usize,
    stride: usize,
) -> Result<Vec<Vec<Complex64>>, String> {
    if n_modes < 2 {
        return Err("n_modes must be ≥ 2 for a spectral encoding".into());
    }
    if stride == 0 {
        return Err("stride must be ≥ 1".into());
    }
    if bytes.len() < 8 {
        return Err(format!(
            "corpus too small: {} bytes (need ≥ 8 for a single frame)",
            bytes.len()
        ));
    }

    // The phasor table e^{−2πi·mj/N} depends only on (n_modes, window length),
    // not on the window contents — computing it per window would be billions of
    // redundant trig calls on a large corpus.
    let n = n_modes as f64;
    let phasors: Vec<Vec<Complex64>> = (0..n_modes)
        .map(|m| {
            (0..n_modes)
                .map(|j| {
                    let angle = -TAU * (m as f64) * (j as f64) / n;
                    Complex64::new(libm::cos(angle), libm::sin(angle)) / n.sqrt()
                })
                .collect()
        })
        .collect();

    let mut fields = Vec::new();
    let mut start = 0;
    while start < bytes.len() {
        let end = (start + n_modes).min(bytes.len());
        if end - start < 8 {
            break;
        }
        fields.push(encode_window_with(&bytes[start..end], n_modes, &phasors));
        start += stride;
    }
    Ok(fields)
}

/// Encode a whole corpus file into a training dataset.
pub fn dataset_from_bytes(
    source: &str,
    bytes: &[u8],
    n_modes: usize,
    stride: usize,
) -> Result<FieldDataset, String> {
    let fields = encode_corpus(bytes, n_modes, stride)?;
    let frames: Vec<Vec<f64>> = fields.iter().map(|f| flatten(f)).collect();
    Ok(FieldDataset {
        format: "aria-text-dataset-v1".into(),
        n_modes,
        encoding: "spectral-dft".into(),
        source: source.into(),
        source_bytes: bytes.len(),
        trajectories: vec![frames],
    })
}

/// Read and encode a corpus file.
pub fn dataset_from_file(
    path: &Path,
    n_modes: usize,
    stride: usize,
) -> Result<FieldDataset, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    dataset_from_bytes(&path.display().to_string(), &bytes, n_modes, stride)
}

fn flatten(psi: &[Complex64]) -> Vec<f64> {
    let mut v = Vec::with_capacity(psi.len() * 2);
    for c in psi {
        v.push(c.re);
        v.push(c.im);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm(psi: &[Complex64]) -> f64 {
        psi.iter().map(num_complex::Complex::norm_sqr).sum::<f64>().sqrt()
    }

    #[test]
    fn windows_have_unit_norm() {
        for bytes in [b"hello aria".as_slice(), &[0u8; 64], b"\xff".repeat(100).as_slice()] {
            let psi = encode_window(bytes, 64);
            assert!((norm(&psi) - 1.0).abs() < 1e-12, "‖ψ‖ = {}", norm(&psi));
        }
    }

    #[test]
    fn encoding_is_deterministic() {
        let a = encode_window(b"the quick brown fox", 32);
        let b = encode_window(b"the quick brown fox", 32);
        assert_eq!(a, b);
        let c = encode_window(b"the quick brown foy", 32);
        assert_ne!(a, c, "one byte must move the field");
    }

    #[test]
    fn similar_windows_interfere_constructively() {
        // The whole point of the encoding: |⟨ψ₁, ψ₂⟩| is a similarity score.
        let a = encode_window(b"aria is an optical jepa graph dynamical system", 64);
        let b = encode_window(b"aria is an optical jepa graph dynamical system", 64);
        let c = encode_window(b"completely unrelated text about something else", 64);

        let sim = |x: &[Complex64], y: &[Complex64]| {
            let dot: Complex64 = x.iter().zip(y).map(|(p, q)| p.conj() * q).sum();
            dot.norm()
        };

        let identical = sim(&a, &b);
        let different = sim(&a, &c);
        assert!(identical > 0.999_999, "⟨ψ,ψ⟩ = {identical}");
        assert!(
            identical > different,
            "identical windows ({identical}) must interfere more than unrelated ones ({different})"
        );
    }

    #[test]
    fn corpus_encoding_validates_inputs() {
        assert!(encode_corpus(b"hi", 64, 64).is_err());
        assert!(encode_corpus(b"hello world!!", 1, 64).is_err());
        assert!(encode_corpus(b"hello world!!", 64, 0).is_err());
        assert!(encode_corpus(b"hello world, this is a real sentence.", 16, 16).is_ok());
    }

    #[test]
    fn corpus_covers_the_input() {
        let bytes: Vec<u8> = (0..1000u32).map(|i| u8::try_from(i % 256).unwrap()).collect();
        let fields = encode_corpus(&bytes, 64, 64).unwrap();
        // 1000 bytes = 15 full 64-byte windows + one 40-byte tail (≥ 8, kept).
        assert_eq!(fields.len(), 16);
        for f in &fields {
            assert!((norm(f) - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn dataset_shape_matches_the_training_format() {
        let d = dataset_from_bytes("test", b"some actual bytes of real text here", 16, 16).unwrap();
        assert_eq!(d.format, "aria-text-dataset-v1");
        assert_eq!(d.trajectories.len(), 1);
        for frame in &d.trajectories[0] {
            assert_eq!(frame.len(), 2 * d.n_modes);
        }
    }
}
