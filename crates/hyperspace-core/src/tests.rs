use super::*;

#[test]
fn test_euclidean_distance() {
    let a = [1.0, 2.0, 3.0];
    let b = [4.0, 5.0, 6.0];
    // diffs: -3, -3, -3. sq: 9, 9, 9. sum: 27.
    let dist = EuclideanMetric::distance(&a, &b);
    assert!((dist - 27.0).abs() < f64::EPSILON);
}

#[test]
fn test_cosine_distance() {
    // Vectors must be normalized for CosineMetric usually, but logic is sum((a-b)^2)
    // If a=[1,0], b=[0,1]. diff=[1, -1]. sq=[1, 1]. sum=2.
    // 2*(1 - cos(90)) = 2*(1-0) = 2. Correct.
    let a = [1.0, 0.0];
    let b = [0.0, 1.0];
    let dist = CosineMetric::distance(&a, &b);
    assert!((dist - 2.0).abs() < f64::EPSILON);

    // a=[1,0], b=[1,0]. diff=[0,0]. sum=0.
    let dist_same = CosineMetric::distance(&a, &a);
    assert!(dist_same.abs() < f64::EPSILON);

    // a=[1,0], b=[-1,0]. diff=[2,0]. sq=[4,0]. sum=4.
    // 2(1 - cos(180)) = 2(1 - (-1)) = 4. Correct.
    let c = [-1.0, 0.0];
    let dist_opp = CosineMetric::distance(&a, &c);
    assert!((dist_opp - 4.0).abs() < f64::EPSILON);
}

#[test]
fn test_poincare_validation() {
    let v_valid = [0.1, 0.2];
    assert!(PoincareMetric::validate(&v_valid).is_ok());

    let v_invalid = [1.0, 0.0]; // Norm=1. Boundary.
    assert!(PoincareMetric::validate(&v_invalid).is_err());

    let v_invalid2 = [0.8, 0.8]; // Norm sq = 0.64+0.64 = 1.28
    assert!(PoincareMetric::validate(&v_invalid2).is_err());
}
