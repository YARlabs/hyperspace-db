import 'dart:math';
import 'dart:typed_data';
import 'dart:convert';
import 'package:crypto/crypto.dart';

/// HyperspaceDB Spatial and Cognitive Math SDK
/// Provides hyperbolic math functions and Cognitive AI metrics for solving LLM hallucinations.

double dot(List<double> a, List<double> b) {
  double sum = 0.0;
  for (int i = 0; i < a.length; i++) sum += a[i] * b[i];
  return sum;
}

double normSq(List<double> v) => dot(v, v);

double norm(List<double> v) => sqrt(max(normSq(v), 0.0));

List<double> projectToBall(List<double> x, double c) {
  double n = norm(x);
  double maxN = (1.0 / sqrt(c)) - 1e-9;
  if (n <= maxN || n <= 1e-15) return List.from(x);
  double s = maxN / n;
  return x.map((v) => v * s).toList();
}

List<double> mobiusAdd(List<double> x, List<double> y, [double c = 1.0]) {
  if (x.length != y.length) throw ArgumentError("Dimension mismatch");
  if (c <= 0.0) throw ArgumentError("Curvature c must be > 0");
  double xy = dot(x, y);
  double x2 = normSq(x);
  double y2 = normSq(y);
  double numLeft = 1.0 + 2.0 * c * xy + c * y2;
  double numRight = 1.0 - c * x2;
  double den = 1.0 + 2.0 * c * xy + c * c * x2 * y2;
  if (den.abs() < 1e-15) throw StateError("Möbius addition denominator too close to zero");
  List<double> res = [];
  for (int i = 0; i < x.length; i++) {
    res.add((numLeft * x[i] + numRight * y[i]) / den);
  }
  return res;
}

List<double> expMap(List<double> x, List<double> v, [double c = 1.0]) {
  if (x.length != v.length) throw ArgumentError("Dimension mismatch");
  if (c <= 0.0) throw ArgumentError("Curvature c must be > 0");
  double x2 = normSq(x);
  double vNorm = norm(v);
  if (vNorm < 1e-15) return List.from(x);
  double lambdaX = 2.0 / max(1.0 - c * x2, 1e-15);
  double scale = tanh(sqrt(c) * lambdaX * vNorm / 2.0) / (sqrt(c) * vNorm);
  List<double> step = v.map((vi) => scale * vi).toList();
  return mobiusAdd(x, step, c);
}

List<double> logMap(List<double> x, List<double> y, [double c = 1.0]) {
  if (x.length != y.length) throw ArgumentError("Dimension mismatch");
  if (c <= 0.0) throw ArgumentError("Curvature c must be > 0");
  List<double> negX = x.map((xi) => -xi).toList();
  List<double> delta = mobiusAdd(negX, y, c);
  double deltaNorm = norm(delta);
  if (deltaNorm < 1e-15) return List.filled(x.length, 0.0);
  double x2 = normSq(x);
  double lambdaX = 2.0 / max(1.0 - c * x2, 1e-15);
  double factor = (2.0 / (lambdaX * sqrt(c))) * atanh(min(sqrt(c) * deltaNorm, 1.0 - 1e-15));
  return delta.map((di) => factor * di / deltaNorm).toList();
}

List<double> gyro(List<double> u, List<double> v, List<double> w, [double c = 1.0]) {
  List<double> uv = mobiusAdd(u, v, c);
  List<double> vw = mobiusAdd(v, w, c);
  List<double> left = mobiusAdd(u, vw, c);
  List<double> negUv = uv.map((z) => -z).toList();
  return mobiusAdd(negUv, left, c);
}

List<double> parallelTransport(List<double> x, List<double> y, List<double> v, [double c = 1.0]) {
  if (x.length != y.length || x.length != v.length) throw ArgumentError("Dimension mismatch");
  if (c <= 0.0) throw ArgumentError("Curvature c must be > 0");
  List<double> negX = x.map((xi) => -xi).toList();
  List<double> gyr = gyro(y, negX, v, c);
  double lambdaX = 2.0 / max(1.0 - c * normSq(x), 1e-15);
  double lambdaY = 2.0 / max(1.0 - c * normSq(y), 1e-15);
  double scale = lambdaX / lambdaY;
  return gyr.map((gi) => scale * gi).toList();
}

List<double> frechetMean(List<List<double>> points, {double c = 1.0, int maxIter = 32, double tol = 1e-6}) {
  if (points.isEmpty) throw ArgumentError("Points set cannot be empty");
  if (c <= 0.0) throw ArgumentError("Curvature c must be > 0");
  int dim = points[0].length;
  List<double> mu = projectToBall(points[0], c);
  for (int iter = 0; iter < max(1, maxIter); iter++) {
    List<double> grad = List.filled(dim, 0.0);
    for (var p in points) {
      List<double> lg = logMap(mu, p, c);
      for (int i = 0; i < dim; i++) grad[i] += lg[i];
    }
    double inv = 1.0 / points.length;
    for (int i = 0; i < dim; i++) grad[i] *= inv;
    double gNorm = norm(grad);
    if (gNorm <= max(tol, 1e-15)) break;
    mu = expMap(mu, grad, c);
    mu = projectToBall(mu, c);
  }
  return mu;
}

double poincareDist(List<double> u, List<double> v, [double c = 1.0]) {
  double u2 = normSq(u);
  double v2 = normSq(v);
  double diff2 = 0.0;
  for (int i = 0; i < u.length; i++) diff2 += pow(u[i] - v[i], 2);
  double denom = (1.0 - c * u2) * (1.0 - c * v2);
  double arg = 1.0 + 2.0 * c * diff2 / max(denom, 1e-15);
  return acosh(arg);
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
  return acosh(max(inner, 1.0));
}

List<double> lorentzToPoincare(List<double> x) {
  if (x.isEmpty) return [];
  double x0 = x[0];
  double denom = max(1.0 + x0, 1e-12);
  return x.skip(1).map((xi) => xi / denom).toList();
}

List<double> poincareToLorentz(List<double> p) {
  double p2 = normSq(p);
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

// Helper Hyperbolic Math Utilities
double tanh(double x) {
  double exp2x = exp(2.0 * x);
  return (exp2x - 1.0) / (exp2x + 1.0);
}

double atanh(double x) {
  return 0.5 * log((1.0 + x) / (1.0 - x));
}

double acosh(double x) {
  return log(x + sqrt(x * x - 1.0));
}

// ==========================================
// Cognitive Math SDK (Spatial AI Engine)
// ==========================================

double localEntropy(List<double> candidate, List<List<double>> neighbors, {double c = 1.0}) {
  if (neighbors.isEmpty) return 1.0;
  double totalDeviation = 0.0;
  for (var neighbor in neighbors) {
    List<double> diff = logMap(candidate, neighbor, c);
    totalDeviation += norm(diff);
  }
  double meanDeviation = totalDeviation / neighbors.length;
  return 1.0 - exp(-meanDeviation);
}

double lyapunovConvergence(List<List<double>> trajectory, {double c = 1.0}) {
  if (trajectory.length < 3) throw ArgumentError("Need at least 3 points");
  List<double> attractor = frechetMean(trajectory, c: c, maxIter: 32, tol: 1e-6);
  double vDiffSum = 0.0;
  for (int i = 0; i < trajectory.length - 1; i++) {
    double vt0 = norm(logMap(attractor, trajectory[i], c));
    double vt1 = norm(logMap(attractor, trajectory[i + 1], c));
    vDiffSum += (vt1 - vt0);
  }
  return vDiffSum / (trajectory.length - 1);
}

List<double> koopmanExtrapolate(List<double> past, List<double> current, double steps, {double c = 1.0}) {
  List<double> velocityAtPast = logMap(past, current, c);
  List<double> velocityAtCurrent = parallelTransport(past, current, velocityAtPast, c);
  List<double> futureVelocity = velocityAtCurrent.map((v) => v * steps).toList();
  return expMap(current, futureVelocity, c);
}

List<double> contextResonance(List<double> thought, List<double> globalContext, double resonanceFactor, {double c = 1.0}) {
  List<double> pullDir = logMap(thought, globalContext, c);
  double factor = max(0.0, min(1.0, resonanceFactor));
  List<double> appliedPull = pullDir.map((v) => v * factor).toList();
  return expMap(thought, appliedPull, c);
}

List<List<double>> generateOrthogonalMatrix(int dimension, List<int> seedBytes) {
  final matrix = List.generate(dimension, (_) => List<double>.filled(dimension, 0.0));
  var currentHash = sha256.convert(seedBytes).bytes;
  var hashOffset = 0;
  final byteData = ByteData(4);

  for (int i = 0; i < dimension; i++) {
    for (int j = 0; j < dimension; j++) {
      if (hashOffset >= 32) {
        currentHash = sha256.convert(currentHash).bytes;
        hashOffset = 0;
      }
      byteData.setUint8(0, currentHash[hashOffset]);
      byteData.setUint8(1, currentHash[hashOffset + 1]);
      byteData.setUint8(2, currentHash[hashOffset + 2]);
      byteData.setUint8(3, currentHash[hashOffset + 3]);
      final valInt = byteData.getUint32(0, Endian.little);
      final val = (valInt / 4294967296.0) * 2.0 - 1.0;
      matrix[i][j] = val;
      hashOffset += 4;
    }
  }

  // Gram-Schmidt with Reorthogonalization
  for (int i = 0; i < dimension; i++) {
    var v = List<double>.from(matrix[i]);
    for (int j = 0; j < i; j++) {
      final u = matrix[j];
      var uDotV = dot(u, v);
      for (int k = 0; k < dimension; k++) {
        v[k] -= uDotV * u[k];
      }
      uDotV = dot(u, v);
      for (int k = 0; k < dimension; k++) {
        v[k] -= uDotV * u[k];
      }
    }
    final vNorm = norm(v);
    if (vNorm > 1e-15) {
      for (int k = 0; k < dimension; k++) {
        matrix[i][k] = v[k] / vNorm;
      }
    } else {
      for (int k = 0; k < dimension; k++) {
        matrix[i][k] = 0.0;
      }
      matrix[i][i] = 1.0;
    }
  }

  return matrix;
}

List<List<double>> generateLorentzMatrix(int dimension, List<int> seedBytes) {
  final d = dimension - 1;

  final rSeed = sha256.convert([...seedBytes, ...utf8.encode("spatial")]).bytes;
  final R = generateOrthogonalMatrix(d, rSeed);

  final bSeed = sha256.convert([...seedBytes, ...utf8.encode("boost")]).bytes;
  var beta = List<double>.filled(d, 0.0);
  var currentHash = bSeed;
  var hashOffset = 0;
  final byteData = ByteData(4);

  for (int i = 0; i < d; i++) {
    if (hashOffset >= 32) {
      currentHash = sha256.convert(currentHash).bytes;
      hashOffset = 0;
    }
    byteData.setUint8(0, currentHash[hashOffset]);
    byteData.setUint8(1, currentHash[hashOffset + 1]);
    byteData.setUint8(2, currentHash[hashOffset + 2]);
    byteData.setUint8(3, currentHash[hashOffset + 3]);
    final valInt = byteData.getUint32(0, Endian.little);
    final val = (valInt / 4294967296.0) * 2.0 - 1.0;
    beta[i] = val;
    hashOffset += 4;
  }

  final betaNorm = norm(beta);
  if (betaNorm > 1e-15) {
    for (int i = 0; i < d; i++) {
      beta[i] = (beta[i] / betaNorm) * 0.1;
    }
  } else {
    for (int i = 0; i < d; i++) {
      beta[i] = 0.0;
    }
    beta[0] = 0.1;
  }

  final betaSq = dot(beta, beta);
  final gamma = 1.0 / sqrt(1.0 - betaSq);

  final Lambda_B = List.generate(dimension, (_) => List<double>.filled(dimension, 0.0));
  Lambda_B[0][0] = gamma;
  for (int j = 1; j < dimension; j++) {
    Lambda_B[0][j] = -gamma * beta[j - 1];
    Lambda_B[j][0] = -gamma * beta[j - 1];
  }

  final factor = (gamma - 1.0) / betaSq;
  for (int i = 1; i < dimension; i++) {
    for (int j = 1; j < dimension; j++) {
      final delta = i == j ? 1.0 : 0.0;
      Lambda_B[i][j] = delta + factor * beta[i - 1] * beta[j - 1];
    }
  }

  final Lambda_R = List.generate(dimension, (_) => List<double>.filled(dimension, 0.0));
  Lambda_R[0][0] = 1.0;
  for (int i = 1; i < dimension; i++) {
    for (int j = 1; j < dimension; j++) {
      Lambda_R[i][j] = R[i - 1][j - 1];
    }
  }

  final Lambda = List.generate(dimension, (_) => List<double>.filled(dimension, 0.0));
  for (int i = 0; i < dimension; i++) {
    for (int j = 0; j < dimension; j++) {
      double sum = 0.0;
      for (int k = 0; k < dimension; k++) {
        sum += Lambda_B[i][k] * Lambda_R[k][j];
      }
      Lambda[i][j] = sum;
    }
  }

  return Lambda;
}

List<double> projectVector(List<double> v, List<List<double>> matrix) {
  final dim = v.length;
  final projected = List<double>.filled(dim, 0.0);
  for (int j = 0; j < dim; j++) {
    double sum = 0.0;
    for (int i = 0; i < dim; i++) {
      sum += v[i] * matrix[i][j];
    }
    projected[j] = sum;
  }
  return projected;
}

List<double> injectAnisotropicNoise(List<double> v, List<int> seedBytes, double sigma) {
  if (v.isEmpty || sigma <= 0.0) return List.from(v);
  final dim = v.length;
  final noise = List<double>.filled(dim, 0.0);
  var currentHash = sha256.convert(seedBytes).bytes;
  var hashOffset = 0;
  final byteData = ByteData(4);

  for (int i = 0; i < dim; i++) {
    if (hashOffset >= 32) {
      currentHash = sha256.convert(currentHash).bytes;
      hashOffset = 0;
    }
    byteData.setUint8(0, currentHash[hashOffset]);
    byteData.setUint8(1, currentHash[hashOffset + 1]);
    byteData.setUint8(2, currentHash[hashOffset + 2]);
    byteData.setUint8(3, currentHash[hashOffset + 3]);
    final valInt = byteData.getUint32(0, Endian.little);
    final val = (valInt / 4294967296.0) * 2.0 - 1.0;
    noise[i] = val;
    hashOffset += 4;
  }

  final vNorm = norm(v);
  if (vNorm > 1e-15) {
    final projLength = dot(noise, v) / (vNorm * vNorm);
    for (int i = 0; i < dim; i++) {
      noise[i] -= projLength * v[i];
    }
  }

  final noiseNorm = norm(noise);
  if (noiseNorm > 1e-15) {
    final scale = sigma * vNorm / noiseNorm;
    return List.generate(dim, (i) => v[i] + noise[i] * scale);
  } else {
    return List.from(v);
  }
}
