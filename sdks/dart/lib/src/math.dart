import 'dart:math';

/// HyperspaceDB Cognitive Math (Spatial AI Engine)
/// Provides offline/client-side tools for Agentic AI in Hyperbolic Space.

/// Calculates the local spatial entropy of a candidate vector based on its distance to neighbors.
/// Returns a value [0.0, 1.0) where 1.0 represents high entropy/chaos (Hallucination).
double localEntropy(List<double> candidate, List<List<double>> neighbors, {double c = 1.0}) {
  if (neighbors.isEmpty) return 1.0;
  
  double totalDeviation = 0.0;
  for (var neighbor in neighbors) {
    // Math logic simplification for Dart without linear algebra module.
    // Replace with true manifold metric distance in hyperbolic.
    double sqDist = _euclideanDist(candidate, neighbor);
    totalDeviation += sqrt(sqDist);
  }
  
  double meanDeviation = totalDeviation / neighbors.length;
  return 1.0 - exp(-meanDeviation);
}

/// Evaluates Lyapunov convergence over a chain of thought.
/// Returns negative values for convergence, positive for divergence.
double lyapunovConvergence(List<List<double>> trajectory, {double c = 1.0}) {
  if (trajectory.length < 3) return 0.0; // Need history

  double vDiffSum = 0.0;
  for (int i = 0; i < trajectory.length - 1; i++) {
    double vT0 = _euclideanDist(trajectory[i], trajectory.last);
    double vT1 = _euclideanDist(trajectory[i + 1], trajectory.last);
    vDiffSum += (vT1 - vT0);
  }
  return vDiffSum / (trajectory.length - 1);
}

double _euclideanDist(List<double> a, List<double> b) {
  double sum = 0.0;
  for (int i = 0; i < a.length; i++) {
    sum += pow(a[i] - b[i], 2);
  }
  return sum;
}
