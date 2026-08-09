use aria_engine_core::engine::OpticalBackend;
use num_complex::Complex64;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::f64::consts::PI;

/// Simulated optical backend — ideal unitary evolution.
///
/// Phase 1 builds one fixed-seed random unitary matrix at construction and
/// reuses it for every OpticalStep. This preserves energy (Inv1) while keeping
/// each step O(N²) instead of O(N³), which is required for the default
/// 1,000-step CLI run at N=256.
#[derive(Debug)]
pub struct SimOptical {
    matrix: Vec<Vec<Complex64>>,
}

impl SimOptical {
    pub fn new(n_modes: usize) -> Self {
        Self::with_seed(n_modes, 42)
    }

    pub fn with_seed(n_modes: usize, seed: u64) -> Self {
        let matrix = make_unitary(n_modes, seed, 0);
        SimOptical { matrix }
    }
}

impl OpticalBackend for SimOptical {
    fn unitary_step(&self, _t: u64, psi: &[Complex64]) -> Vec<Complex64> {
        mat_vec_mul(&self.matrix, psi)
    }

    fn energy(&self, psi: &[Complex64]) -> f64 {
        psi.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt()
    }
}

/// Build a deterministic N×N unitary matrix as a product of N Householder
/// reflections and diagonal phase rotations.
///
/// Each reflection `H = I − 2vv†` is applied as a rank-1 update rather than by
/// forming `H` and multiplying: `H·U = U − 2v(v†U)` costs O(N²), so the whole
/// product costs O(N³) instead of the O(N⁴) an explicit matmul per reflection
/// would take. At N = 256 that is the difference between ~8 s and ~30 ms of
/// setup.
fn make_unitary(n_modes: usize, seed: u64, t: u64) -> Vec<Vec<Complex64>> {
    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(t));
    let n = n_modes;

    // Start with the identity.
    let mut u: Vec<Vec<Complex64>> = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| if i == j { Complex64::new(1.0, 0.0) } else { Complex64::ZERO })
                .collect()
        })
        .collect();

    let mut w = vec![Complex64::ZERO; n];

    for _k in 0..n {
        // A random unit vector v on the complex unit sphere.
        let mut v: Vec<Complex64> = (0..n)
            .map(|_| {
                let theta: f64 = rng.gen::<f64>() * 2.0 * PI;
                Complex64::new(theta.cos(), theta.sin()) / (n as f64).sqrt()
            })
            .collect();
        let norm: f64 = v.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt();
        for vi in &mut v {
            *vi /= Complex64::new(norm, 0.0);
        }

        // w = v† U   (row vector, one entry per column of U)
        w.iter_mut().for_each(|x| *x = Complex64::ZERO);
        for (i, row) in u.iter().enumerate() {
            let vi_conj = v[i].conj();
            for (wj, uij) in w.iter_mut().zip(row) {
                *wj += vi_conj * uij;
            }
        }

        // U ← phase · (U − 2 v w)
        let phase: f64 = rng.gen::<f64>() * 2.0 * PI;
        let phase_c = Complex64::new(phase.cos(), phase.sin());
        for (i, row) in u.iter_mut().enumerate() {
            let two_vi = Complex64::new(2.0, 0.0) * v[i];
            for (uij, wj) in row.iter_mut().zip(&w) {
                *uij = phase_c * (*uij - two_vi * wj);
            }
        }
    }

    u
}

fn mat_vec_mul(m: &[Vec<Complex64>], v: &[Complex64]) -> Vec<Complex64> {
    let n = v.len();
    let mut result = vec![Complex64::ZERO; n];
    for i in 0..n {
        for j in 0..n {
            result[i] += m[i][j] * v[j];
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn energy_conserved() {
        let opt = SimOptical::new(8);
        let psi0: Vec<Complex64> = (0..8)
            .map(|i| Complex64::new((i as f64).cos(), (i as f64).sin()))
            .collect();
        let e0 = opt.energy(&psi0);

        for t in 0..10 {
            let psi1 = opt.unitary_step(t, &psi0);
            let e1 = opt.energy(&psi1);
            // Different psi (rotated) but same energy
            assert!((e1 - e0).abs() < 1e-10, "energy not conserved at t={}", t);
            // Not identity
            assert!(psi1 != psi0 || t > 0, "unitary is identity — unlikely");
        }
    }

    #[test]
    fn matrix_is_unitary() {
        // U†U = I to f64 precision — this is what makes Inv1 hold exactly.
        let n = 24;
        let u = make_unitary(n, 7, 0);
        for i in 0..n {
            for j in 0..n {
                let dot: Complex64 = (0..n).map(|k| u[k][i].conj() * u[k][j]).sum();
                let want = if i == j { 1.0 } else { 0.0 };
                assert!(
                    (dot.re - want).abs() < 1e-9 && dot.im.abs() < 1e-9,
                    "U†U[{i}][{j}] = {dot}, want {want}"
                );
            }
        }
    }

    #[test]
    fn seed_changes_matrix() {
        let a = SimOptical::with_seed(8, 1);
        let b = SimOptical::with_seed(8, 2);
        let psi0: Vec<Complex64> = (0..8)
            .map(|i| Complex64::new((i as f64).cos(), (i as f64).sin()))
            .collect();
        let psi_a = a.unitary_step(0, &psi0);
        let psi_b = b.unitary_step(0, &psi0);
        assert_ne!(psi_a, psi_b, "different seeds should produce different unitaries");
    }
}
