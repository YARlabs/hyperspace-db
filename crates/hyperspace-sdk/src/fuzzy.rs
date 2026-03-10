#![allow(clippy::manual_range_contains)]
//! Fuzzy set-theoretic operators: t-norms, t-conorms, and negation.
//!
//! These operators extend crisp set operations (intersection, union, complement)
//! to fuzzy membership values in \[0, 1\]. They are the foundation of fuzzy
//! query answering over knowledge graphs.
//!
//! # T-norms (fuzzy intersection)
//!
//! A t-norm `T: [0,1]^2 -> [0,1]` must satisfy:
//! - Commutativity: `T(a, b) = T(b, a)`
//! - Associativity: `T(a, T(b, c)) = T(T(a, b), c)`
//! - Monotonicity: if `a <= c` then `T(a, b) <= T(c, b)`
//! - Identity: `T(a, 1) = a`
//! - Annihilator: `T(a, 0) = 0`
//!
//! # T-conorms (fuzzy union)
//!
//! Each t-norm has a dual t-conorm via De Morgan's law:
//! `S(a, b) = 1 - T(1-a, 1-b)`.
//!
//! # References
//!
//! - Chen et al. (AAAI 2022), "Fuzzy Logic Based Logical Query Answering on
//!   Knowledge Graphs" (`FuzzQE`)
//!
//! # Examples
//!
//! ```rust
//! use hyperspace_sdk::fuzzy::{TNorm, TConorm, fuzzy_negation};
//!
//! // Fuzzy intersection: how "aquatic" AND "mammal" is a dolphin?
//! let aquatic = 0.9;
//! let mammal = 0.95;
//!
//! let min = TNorm::Min.apply(aquatic, mammal);       // 0.9
//! let prod = TNorm::Product.apply(aquatic, mammal);   // 0.855
//! let luk = TNorm::Lukasiewicz.apply(aquatic, mammal); // 0.85
//! assert!(min >= prod && prod >= luk); // Min >= Product >= Lukasiewicz
//!
//! // De Morgan duality: neg(T(a,b)) = S(neg(a), neg(b))
//! let t = TNorm::Product;
//! let s = t.dual(); // TConorm::Probabilistic
//! let lhs = fuzzy_negation(t.apply(0.7, 0.4));
//! let rhs = s.apply(fuzzy_negation(0.7), fuzzy_negation(0.4));
//! assert!((lhs - rhs).abs() < 1e-6);
//! ```

/// Godel t-norm: `min(a, b)`.
///
/// The strongest t-norm: `min(a, b) >= T(a, b)` for any t-norm T.
#[inline]
#[must_use]
pub fn tnorm_min(a: f32, b: f32) -> f32 {
    a.min(b)
}

/// Product t-norm: `a * b`.
///
/// Corresponds to probabilistic independence: if two events are independent,
/// the probability of both occurring is the product of their probabilities.
#[inline]
#[must_use]
pub fn tnorm_product(a: f32, b: f32) -> f32 {
    a * b
}

/// Lukasiewicz t-norm: `max(a + b - 1, 0)`.
///
/// The weakest common t-norm. Produces 0 unless both inputs are high.
#[inline]
#[must_use]
pub fn tnorm_lukasiewicz(a: f32, b: f32) -> f32 {
    (a + b - 1.0).max(0.0)
}

/// Godel t-conorm (dual of min): `max(a, b)`.
#[inline]
#[must_use]
pub fn tconorm_max(a: f32, b: f32) -> f32 {
    a.max(b)
}

/// Probabilistic t-conorm (dual of product): `a + b - a*b`.
#[inline]
#[must_use]
pub fn tconorm_probabilistic(a: f32, b: f32) -> f32 {
    a + b - a * b
}

/// Lukasiewicz t-conorm (dual of Lukasiewicz): `min(a + b, 1)`.
#[inline]
#[must_use]
pub fn tconorm_lukasiewicz(a: f32, b: f32) -> f32 {
    (a + b).min(1.0)
}

/// Standard fuzzy negation: `1 - a`.
#[inline]
#[must_use]
pub fn fuzzy_negation(a: f32) -> f32 {
    1.0 - a
}

/// A triangular norm (t-norm) for fuzzy intersection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TNorm {
    /// Godel t-norm: `min(a, b)`.
    Min,
    /// Product t-norm: `a * b`.
    Product,
    /// Lukasiewicz t-norm: `max(a + b - 1, 0)`.
    Lukasiewicz,
}

impl TNorm {
    /// Apply this t-norm to two fuzzy values.
    #[inline]
    #[must_use]
    pub fn apply(&self, a: f32, b: f32) -> f32 {
        match self {
            TNorm::Min => tnorm_min(a, b),
            TNorm::Product => tnorm_product(a, b),
            TNorm::Lukasiewicz => tnorm_lukasiewicz(a, b),
        }
    }

    /// Get the dual t-conorm (via De Morgan).
    #[inline]
    #[must_use]
    pub fn dual(&self) -> TConorm {
        match self {
            TNorm::Min => TConorm::Max,
            TNorm::Product => TConorm::Probabilistic,
            TNorm::Lukasiewicz => TConorm::Lukasiewicz,
        }
    }
}

/// A triangular conorm (t-conorm) for fuzzy union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TConorm {
    /// Godel t-conorm: `max(a, b)`.
    Max,
    /// Probabilistic t-conorm: `a + b - a*b`.
    Probabilistic,
    /// Lukasiewicz t-conorm: `min(a + b, 1)`.
    Lukasiewicz,
}

impl TConorm {
    /// Apply this t-conorm to two fuzzy values.
    #[inline]
    #[must_use]
    pub fn apply(&self, a: f32, b: f32) -> f32 {
        match self {
            TConorm::Max => tconorm_max(a, b),
            TConorm::Probabilistic => tconorm_probabilistic(a, b),
            TConorm::Lukasiewicz => tconorm_lukasiewicz(a, b),
        }
    }

    /// Get the dual t-norm (via De Morgan).
    #[inline]
    #[must_use]
    pub fn dual(&self) -> TNorm {
        match self {
            TConorm::Max => TNorm::Min,
            TConorm::Probabilistic => TNorm::Product,
            TConorm::Lukasiewicz => TNorm::Lukasiewicz,
        }
    }
}

/// Represents a fuzzy logic search query over vector space.
#[derive(Debug, Clone)]
pub enum FuzzyQuery {
    /// A single target vector.
    Vector(Vec<f64>),
    /// Fuzzy intersection (AND) using a t-norm.
    And(Box<FuzzyQuery>, Box<FuzzyQuery>, TNorm),
    /// Fuzzy union (OR) using a t-conorm.
    Or(Box<FuzzyQuery>, Box<FuzzyQuery>, TConorm),
    /// Fuzzy negation (NOT).
    Not(Box<FuzzyQuery>),
}

impl FuzzyQuery {
    /// Flatten and extract all vectors in the query graph to a list.
    pub fn extract_vectors(&self, out: &mut Vec<Vec<f64>>) {
        match self {
            FuzzyQuery::Vector(v) => out.push(v.clone()),
            FuzzyQuery::And(lhs, rhs, _) | FuzzyQuery::Or(lhs, rhs, _) => {
                lhs.extract_vectors(out);
                rhs.extract_vectors(out);
            }
            FuzzyQuery::Not(expr) => expr.extract_vectors(out),
        }
    }

    /// Evaluates the `FuzzyQuery` for a database item yielding a final membership score \[0, 1\].
    /// `distance_fn` provides the distance from the database item to a specific query node via its traversal index.
    pub fn evaluate_indexed<F>(&self, counter: &mut usize, distance_fn: &F) -> f32
    where
        F: Fn(usize) -> f32,
    {
        match self {
            FuzzyQuery::Vector(_) => {
                let idx = *counter;
                *counter += 1;
                let dist = distance_fn(idx);
                // Convert distance to fuzzy membership [0, 1].
                (-dist).exp()
            }
            FuzzyQuery::And(lhs, rhs, tnorm) => tnorm.apply(
                lhs.evaluate_indexed(counter, distance_fn),
                rhs.evaluate_indexed(counter, distance_fn),
            ),
            FuzzyQuery::Or(lhs, rhs, tconorm) => tconorm.apply(
                lhs.evaluate_indexed(counter, distance_fn),
                rhs.evaluate_indexed(counter, distance_fn),
            ),
            FuzzyQuery::Not(expr) => fuzzy_negation(expr.evaluate_indexed(counter, distance_fn)),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use proptest::prelude::*;

    // ---- Unit tests ----

    #[test]
    fn tnorm_min_basic() {
        assert_eq!(tnorm_min(0.3, 0.7), 0.3);
        assert_eq!(tnorm_min(0.5, 0.5), 0.5);
        assert_eq!(tnorm_min(1.0, 0.4), 0.4);
        assert_eq!(tnorm_min(0.0, 0.9), 0.0);
    }

    #[test]
    fn tnorm_product_basic() {
        assert!((tnorm_product(0.5, 0.5) - 0.25).abs() < 1e-7);
        assert_eq!(tnorm_product(1.0, 0.7), 0.7);
        assert_eq!(tnorm_product(0.0, 0.5), 0.0);
    }

    #[test]
    fn tnorm_lukasiewicz_basic() {
        assert_eq!(tnorm_lukasiewicz(0.3, 0.5), 0.0); // 0.3+0.5-1 = -0.2 -> 0
        assert!((tnorm_lukasiewicz(0.8, 0.9) - 0.7).abs() < 1e-7);
        assert_eq!(tnorm_lukasiewicz(1.0, 1.0), 1.0);
        assert_eq!(tnorm_lukasiewicz(0.0, 1.0), 0.0);
    }

    #[test]
    fn tconorm_max_basic() {
        assert_eq!(tconorm_max(0.3, 0.7), 0.7);
        assert_eq!(tconorm_max(0.0, 0.0), 0.0);
        assert_eq!(tconorm_max(1.0, 0.5), 1.0);
    }

    #[test]
    fn tconorm_probabilistic_basic() {
        // 0.3 + 0.7 - 0.21 = 0.79
        assert!((tconorm_probabilistic(0.3, 0.7) - 0.79).abs() < 1e-6);
        assert_eq!(tconorm_probabilistic(0.0, 0.5), 0.5);
        assert_eq!(tconorm_probabilistic(1.0, 0.5), 1.0);
    }

    #[test]
    fn tconorm_lukasiewicz_basic() {
        assert!((tconorm_lukasiewicz(0.3, 0.5) - 0.8).abs() < 1e-7);
        assert_eq!(tconorm_lukasiewicz(0.6, 0.7), 1.0); // 1.3 clamped
        assert_eq!(tconorm_lukasiewicz(0.0, 0.0), 0.0);
    }

    #[test]
    fn fuzzy_negation_basic() {
        assert_eq!(fuzzy_negation(0.0), 1.0);
        assert_eq!(fuzzy_negation(1.0), 0.0);
        assert!((fuzzy_negation(0.3) - 0.7).abs() < 1e-7);
    }

    #[test]
    fn enum_apply() {
        assert_eq!(TNorm::Min.apply(0.3, 0.7), tnorm_min(0.3, 0.7));
        assert_eq!(TNorm::Product.apply(0.3, 0.7), tnorm_product(0.3, 0.7));
        assert_eq!(
            TNorm::Lukasiewicz.apply(0.3, 0.7),
            tnorm_lukasiewicz(0.3, 0.7)
        );
        assert_eq!(TConorm::Max.apply(0.3, 0.7), tconorm_max(0.3, 0.7));
        assert_eq!(
            TConorm::Probabilistic.apply(0.3, 0.7),
            tconorm_probabilistic(0.3, 0.7)
        );
        assert_eq!(
            TConorm::Lukasiewicz.apply(0.3, 0.7),
            tconorm_lukasiewicz(0.3, 0.7)
        );
    }

    #[test]
    fn dual_roundtrip() {
        assert_eq!(TNorm::Min.dual().dual(), TNorm::Min);
        assert_eq!(TNorm::Product.dual().dual(), TNorm::Product);
        assert_eq!(TNorm::Lukasiewicz.dual().dual(), TNorm::Lukasiewicz);
    }

    // ---- Property tests ----

    fn arb_unit() -> impl Strategy<Value = f32> {
        0.0f32..=1.0
    }

    proptest! {
        // -- Commutativity --

        #[test]
        fn prop_tnorm_min_commutative(a in arb_unit(), b in arb_unit()) {
            prop_assert_eq!(tnorm_min(a, b), tnorm_min(b, a));
        }

        #[test]
        fn prop_tnorm_product_commutative(a in arb_unit(), b in arb_unit()) {
            prop_assert!((tnorm_product(a, b) - tnorm_product(b, a)).abs() < 1e-6);
        }

        #[test]
        fn prop_tnorm_lukasiewicz_commutative(a in arb_unit(), b in arb_unit()) {
            prop_assert!((tnorm_lukasiewicz(a, b) - tnorm_lukasiewicz(b, a)).abs() < 1e-6);
        }

        // -- Associativity --

        #[test]
        fn prop_tnorm_min_associative(a in arb_unit(), b in arb_unit(), c in arb_unit()) {
            let lhs = tnorm_min(a, tnorm_min(b, c));
            let rhs = tnorm_min(tnorm_min(a, b), c);
            prop_assert!((lhs - rhs).abs() < 1e-6);
        }

        #[test]
        fn prop_tnorm_product_associative(a in arb_unit(), b in arb_unit(), c in arb_unit()) {
            let lhs = tnorm_product(a, tnorm_product(b, c));
            let rhs = tnorm_product(tnorm_product(a, b), c);
            prop_assert!((lhs - rhs).abs() < 1e-5);
        }

        #[test]
        fn prop_tnorm_lukasiewicz_associative(a in arb_unit(), b in arb_unit(), c in arb_unit()) {
            let lhs = tnorm_lukasiewicz(a, tnorm_lukasiewicz(b, c));
            let rhs = tnorm_lukasiewicz(tnorm_lukasiewicz(a, b), c);
            prop_assert!((lhs - rhs).abs() < 1e-6);
        }

        // -- Boundary conditions --

        #[test]
        fn prop_tnorm_identity(a in arb_unit()) {
            // T(a, 1) = a for all t-norms.
            prop_assert!((TNorm::Min.apply(a, 1.0) - a).abs() < 1e-6);
            prop_assert!((TNorm::Product.apply(a, 1.0) - a).abs() < 1e-6);
            prop_assert!((TNorm::Lukasiewicz.apply(a, 1.0) - a).abs() < 1e-6);
        }

        #[test]
        fn prop_tnorm_annihilator(a in arb_unit()) {
            // T(a, 0) = 0 for all t-norms.
            prop_assert!((TNorm::Min.apply(a, 0.0)).abs() < 1e-6);
            prop_assert!((TNorm::Product.apply(a, 0.0)).abs() < 1e-6);
            prop_assert!((TNorm::Lukasiewicz.apply(a, 0.0)).abs() < 1e-6);
        }

        // -- De Morgan: neg(T(a,b)) = S(neg(a), neg(b)) --

        #[test]
        fn prop_de_morgan_min(a in arb_unit(), b in arb_unit()) {
            let lhs = fuzzy_negation(tnorm_min(a, b));
            let rhs = tconorm_max(fuzzy_negation(a), fuzzy_negation(b));
            prop_assert!((lhs - rhs).abs() < 1e-6,
                "De Morgan min: {lhs} != {rhs}");
        }

        #[test]
        fn prop_de_morgan_product(a in arb_unit(), b in arb_unit()) {
            let lhs = fuzzy_negation(tnorm_product(a, b));
            let rhs = tconorm_probabilistic(fuzzy_negation(a), fuzzy_negation(b));
            prop_assert!((lhs - rhs).abs() < 1e-5,
                "De Morgan product: {lhs} != {rhs}");
        }

        #[test]
        fn prop_de_morgan_lukasiewicz(a in arb_unit(), b in arb_unit()) {
            let lhs = fuzzy_negation(tnorm_lukasiewicz(a, b));
            let rhs = tconorm_lukasiewicz(fuzzy_negation(a), fuzzy_negation(b));
            prop_assert!((lhs - rhs).abs() < 1e-6,
                "De Morgan Lukasiewicz: {lhs} != {rhs}");
        }

        // -- Output range --

        #[test]
        fn prop_tnorm_output_in_unit(a in arb_unit(), b in arb_unit()) {
            for t in [TNorm::Min, TNorm::Product, TNorm::Lukasiewicz] {
                let v = t.apply(a, b);
                prop_assert!(v >= -1e-7 && v <= 1.0 + 1e-7,
                    "t-norm {:?} output {v} out of [0,1]", t);
            }
        }

        #[test]
        fn prop_tconorm_output_in_unit(a in arb_unit(), b in arb_unit()) {
            for s in [TConorm::Max, TConorm::Probabilistic, TConorm::Lukasiewicz] {
                let v = s.apply(a, b);
                prop_assert!(v >= -1e-7 && v <= 1.0 + 1e-7,
                    "t-conorm {:?} output {v} out of [0,1]", s);
            }
        }

        // -- Double negation --

        #[test]
        fn prop_double_negation(a in arb_unit()) {
            let result = fuzzy_negation(fuzzy_negation(a));
            prop_assert!((result - a).abs() < 1e-6, "double negation: {result} != {a}");
        }
    }
}
