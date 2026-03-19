import 'dart:math';

/// HyperspaceDB Cognitive Math (Spatial AI Engine)
/// Provides offline/client-side tools for Agentic AI in Hyperbolic Space.

// ==========================================
// Poincaré Ball Math
// ==========================================

double _dot(List<double> a, List<double> b) {
  double sum = 0.0;
  for (int i = 0; i < a.length; i++) sum += a[i] * b[i];
  return sum;
}

double _normSq(List<double> v) => _dot(v, v);

double poincareDist(List<double> u, List<double> v) {
  double u2 = _normSq(u);
  double v2 = _normSq(v);
  double diff2 = 0.0;
  for (int i = 0; i < u.length; i++) diff2 += pow(u[i] - v[i], 2);
  
  double denom = (1.0 - u2) * (1.0 - v2);
  double arg = 1.0 + 2.0 * diff2 / max(denom, 1e-15);
  return log(arg + sqrt(arg * arg - 1.0)); // acosh(arg)
}

// ==========================================
// Lorentz Model Math (Hyperboloid)
// ==========================================

double lorentzProduct(List<double> u, List<double> v) {
  if (u.isEmpty || v.isEmpty) return 0.0;
  double product = -u[0] * v[0];
  for (int i = 1; i < u.length; i++) product += u[i] * v[i];
  return product;
}

double lorentzDist(List<double> u, List<double> v) {
  double inner = -lorentzProduct(u, v);
  double arg = max(inner, 1.0);
  return log(arg + sqrt(arg * arg - 1.0)); // acosh(arg)
}

List<double> lorentzToPoincare(List<double> x) {
  if (x.isEmpty) return [];
  double x0 = x[0];
  double denom = max(1.0 + x0, 1e-12);
  return x.skip(1).map((xi) => xi / denom).toList();
}

List<double> poincareToLorentz(List<double> p) {
  double p2 = _normSq(p);
  double denom = max(1.0 - p2, 1e-12);
  List<double> x = [(1.0 + p2) / denom];
  for (var pi in p) {
    x.add(2.0 * pi / denom);
  }
  return x;
}

List<double> projectToHyperboloid(List<double> v) {
  if (v.isEmpty) return [];
  List<double> res = List.from(v);
  double spatialNormSq = 0.0;
  for (int i = 1; i < res.length; i++) spatialNormSq += res[i] * res[i];
  res[0] = sqrt(1.0 + spatialNormSq);
  return res;
}

// ==========================================
// Cognitive Math SDK
// ==========================================

double localEntropy(List<double> candidate, List<List<double>> neighbors, {double c = 1.0}) {
  if (neighbors.isEmpty) return 1.0;
  double totalDist = 0.0;
  for (var neighbor in neighbors) {
    totalDist += poincareDist(candidate, neighbor);
  }
  double meanDist = totalDist / neighbors.length;
  return 1.0 - exp(-meanDist);
}

double lyapunovConvergence(List<List<double>> trajectory, {double c = 1.0}) {
  if (trajectory.length < 3) return 0.0;
  double vDiffSum = 0.0;
  for (int i = 0; i < trajectory.length - 1; i++) {
    double vT0 = poincareDist(trajectory[i], trajectory.last);
    double vT1 = poincareDist(trajectory[i + 1], trajectory.last);
    vDiffSum += (vT1 - vT0);
  }
  return vDiffSum / (trajectory.length - 1);
}
