package hyperspace

import (
	"crypto/sha256"
	"encoding/binary"
	"errors"
	"math"
)

// LorentzProduct computes the Minkowski inner product (Lorentz product) between two vectors.
func LorentzProduct(u, v []float64) float64 {
	if len(u) == 0 || len(v) == 0 {
		return 0.0
	}
	dot := -u[0] * v[0]
	for i := 1; i < len(u); i++ {
		dot += u[i] * v[i]
	}
	return dot
}

// LorentzDist computes the Lorentz distance between two points on the hyperboloid.
func LorentzDist(u, v []float64) float64 {
	inner := -LorentzProduct(u, v)
	if inner < 1.0 {
		inner = 1.0
	}
	return math.Acosh(inner)
}

// LorentzToPoincare converts a point from the Lorentz model (Hyperboloid) to the Poincaré Ball model (129 -> 128).
func LorentzToPoincare(x []float64) []float64 {
	if len(x) == 0 {
		return []float64{}
	}
	denom := 1.0 + x[0]
	if denom < 1e-12 {
		denom = 1e-12
	}
	proj := make([]float64, len(x)-1)
	for i := 1; i < len(x); i++ {
		proj[i-1] = x[i] / denom
	}
	return proj
}

// PoincareToLorentz converts a point from the Poincaré Ball model to the Lorentz model (128 -> 129).
func PoincareToLorentz(p []float64) []float64 {
	pSq := 0.0
	for _, v := range p {
		pSq += v * v
	}
	denom := 1.0 - pSq
	if denom < 1e-12 {
		denom = 1e-12
	}
	x := make([]float64, len(p)+1)
	x[0] = (1.0 + pSq) / denom
	for i, pi := range p {
		x[i+1] = 2.0 * pi / denom
	}
	return x
}

// ProjectToHyperboloid ensures a vector satisfies the Lorentz constraint -x0^2 + |x|^2 = -1.
func ProjectToHyperboloid(v []float64) []float64 {
	if len(v) == 0 {
		return []float64{}
	}
	res := make([]float64, len(v))
	copy(res, v)
	spatialNormSq := 0.0
	for i := 1; i < len(res); i++ {
		spatialNormSq += res[i] * res[i]
	}
	res[0] = math.Sqrt(1.0 + spatialNormSq)
	return res
}

// dot calculates the Euclidean dot product.
func dot(u, v []float64) float64 {
	sum := 0.0
	for i := 0; i < len(u); i++ {
		sum += u[i] * v[i]
	}
	return sum
}

// normSq calculates the squared L2 norm.
func normSq(v []float64) float64 {
	return dot(v, v)
}

// norm calculates the L2 norm.
func norm(v []float64) float64 {
	return math.Sqrt(math.Max(normSq(v), 0.0))
}

// projectToBall projects a vector to the Poincaré Ball of curvature c.
func projectToBall(x []float64, c float64) []float64 {
	n := norm(x)
	maxN := (1.0 / math.Sqrt(c)) - 1e-9
	if n <= maxN || n <= 1e-15 {
		res := make([]float64, len(x))
		copy(res, x)
		return res
	}
	s := maxN / n
	res := make([]float64, len(x))
	for i, v := range x {
		res[i] = v * s
	}
	return res
}

// PoincareDist computes the Poincaré distance between two points in the Poincaré Ball.
func PoincareDist(u, v []float64, c float64) float64 {
	u2 := normSq(u)
	v2 := normSq(v)
	diff2 := 0.0
	for i := 0; i < len(u); i++ {
		d := u[i] - v[i]
		diff2 += d * d
	}
	denom := (1.0 - c*u2) * (1.0 - c*v2)
	if denom < 1e-15 {
		denom = 1e-15
	}
	arg := 1.0 + 2.0*c*diff2/denom
	return math.Acosh(arg)
}

// MobiusAdd performs Möbius addition in the Poincaré Ball.
func MobiusAdd(x, y []float64, c float64) ([]float64, error) {
	if len(x) != len(y) {
		return nil, errors.New("dimension mismatch")
	}
	if c <= 0.0 {
		return nil, errors.New("curvature c must be > 0")
	}
	xy := dot(x, y)
	x2 := normSq(x)
	y2 := normSq(y)
	numLeft := 1.0 + 2.0*c*xy + c*y2
	numRight := 1.0 - c*x2
	den := 1.0 + 2.0*c*xy + c*c*x2*y2
	if math.Abs(den) < 1e-15 {
		return nil, errors.New("Möbius addition denominator too close to zero")
	}
	res := make([]float64, len(x))
	for i := 0; i < len(x); i++ {
		res[i] = (numLeft*x[i] + numRight*y[i]) / den
	}
	return res, nil
}

// ExpMap computes the exponential map from a point x in the Poincaré Ball.
func ExpMap(x, v []float64, c float64) ([]float64, error) {
	if len(x) != len(v) {
		return nil, errors.New("dimension mismatch")
	}
	if c <= 0.0 {
		return nil, errors.New("curvature c must be > 0")
	}
	x2 := normSq(x)
	vNorm := norm(v)
	if vNorm < 1e-15 {
		res := make([]float64, len(x))
		copy(res, x)
		return res, nil
	}
	lambdaX := 2.0 / math.Max(1.0-c*x2, 1e-15)
	scale := math.Tanh(math.Sqrt(c)*lambdaX*vNorm/2.0) / (math.Sqrt(c) * vNorm)
	step := make([]float64, len(v))
	for i, vi := range v {
		step[i] = scale * vi
	}
	return MobiusAdd(x, step, c)
}

// LogMap computes the logarithmic map of y from point x in the Poincaré Ball.
func LogMap(x, y []float64, c float64) ([]float64, error) {
	if len(x) != len(y) {
		return nil, errors.New("dimension mismatch")
	}
	if c <= 0.0 {
		return nil, errors.New("curvature c must be > 0")
	}
	negX := make([]float64, len(x))
	for i, xi := range x {
		negX[i] = -xi
	}
	delta, err := MobiusAdd(negX, y, c)
	if err != nil {
		return nil, err
	}
	deltaNorm := norm(delta)
	if deltaNorm < 1e-15 {
		return make([]float64, len(x)), nil
	}
	x2 := normSq(x)
	lambdaX := 2.0 / math.Max(1.0-c*x2, 1e-15)
	factor := (2.0 / (lambdaX * math.Sqrt(c))) * math.Atanh(math.Min(math.Sqrt(c)*deltaNorm, 1.0-1e-15))
	res := make([]float64, len(x))
	for i, di := range delta {
		res[i] = factor * di / deltaNorm
	}
	return res, nil
}

// gyro computes the gyration operator.
func gyro(u, v, w []float64, c float64) ([]float64, error) {
	uv, err := MobiusAdd(u, v, c)
	if err != nil {
		return nil, err
	}
	vw, err := MobiusAdd(v, w, c)
	if err != nil {
		return nil, err
	}
	left, err := MobiusAdd(u, vw, c)
	if err != nil {
		return nil, err
	}
	negUv := make([]float64, len(uv))
	for i, z := range uv {
		negUv[i] = -z
	}
	return MobiusAdd(negUv, left, c)
}

// ParallelTransport transports vector v from x to y in the Poincaré Ball.
func ParallelTransport(x, y, v []float64, c float64) ([]float64, error) {
	if len(x) != len(y) || len(x) != len(v) {
		return nil, errors.New("dimension mismatch")
	}
	if c <= 0.0 {
		return nil, errors.New("curvature c must be > 0")
	}
	negX := make([]float64, len(x))
	for i, xi := range x {
		negX[i] = -xi
	}
	gyr, err := gyro(y, negX, v, c)
	if err != nil {
		return nil, err
	}
	lambdaX := 2.0 / math.Max(1.0-c*normSq(x), 1e-15)
	lambdaY := 2.0 / math.Max(1.0-c*normSq(y), 1e-15)
	scale := lambdaX / lambdaY
	res := make([]float64, len(gyr))
	for i, gi := range gyr {
		res[i] = scale * gi
	}
	return res, nil
}

// FrechetMean computes the Fréchet mean of a set of points in the Poincaré Ball.
func FrechetMean(points [][]float64, c float64, maxIter int, tol float64) ([]float64, error) {
	if len(points) == 0 {
		return nil, errors.New("points set cannot be empty")
	}
	if c <= 0.0 {
		return nil, errors.New("curvature c must be > 0")
	}
	dim := len(points[0])
	mu := projectToBall(points[0], c)
	for iter := 0; iter < maxIter; iter++ {
		grad := make([]float64, dim)
		for _, p := range points {
			lg, err := LogMap(mu, p, c)
			if err != nil {
				return nil, err
			}
			for i := 0; i < dim; i++ {
				grad[i] += lg[i]
			}
		}
		inv := 1.0 / float64(len(points))
		for i := 0; i < dim; i++ {
			grad[i] *= inv
		}
		if norm(grad) <= tol {
			break
		}
		var err error
		mu, err = ExpMap(mu, grad, c)
		if err != nil {
			return nil, err
		}
		mu = projectToBall(mu, c)
	}
	return mu, nil
}

// LocalEntropy calculates the spatial entropy (dispersion) of candidate relative to neighbors.
func LocalEntropy(candidate []float64, neighbors [][]float64, c float64) (float64, error) {
	if len(neighbors) == 0 {
		return 1.0, nil
	}
	totalDeviation := 0.0
	for _, neighbor := range neighbors {
		diff, err := LogMap(candidate, neighbor, c)
		if err != nil {
			return 0.0, err
		}
		totalDeviation += norm(diff)
	}
	meanDeviation := totalDeviation / float64(len(neighbors))
	return 1.0 - math.Exp(-meanDeviation), nil
}

// LyapunovConvergence evaluates convergence trend of a trajectory to an attractor.
func LyapunovConvergence(trajectory [][]float64, c float64) (float64, error) {
	if len(trajectory) < 3 {
		return 0.0, errors.New("need at least 3 points")
	}
	attractor, err := FrechetMean(trajectory, c, 32, 1e-6)
	if err != nil {
		return 0.0, err
	}
	vDiffSum := 0.0
	for i := 0; i < len(trajectory)-1; i++ {
		t0, err := LogMap(attractor, trajectory[i], c)
		if err != nil {
			return 0.0, err
		}
		t1, err := LogMap(attractor, trajectory[i+1], c)
		if err != nil {
			return 0.0, err
		}
		vDiffSum += norm(t1) - norm(t0)
	}
	return vDiffSum / float64(len(trajectory)-1), nil
}

// KoopmanExtrapolate extrapolates trajectory forward.
func KoopmanExtrapolate(past, current []float64, steps float64, c float64) ([]float64, error) {
	velocityAtPast, err := LogMap(past, current, c)
	if err != nil {
		return nil, err
	}
	velocityAtCurrent, err := ParallelTransport(past, current, velocityAtPast, c)
	if err != nil {
		return nil, err
	}
	futureVelocity := make([]float64, len(velocityAtCurrent))
	for i, v := range velocityAtCurrent {
		futureVelocity[i] = v * steps
	}
	return ExpMap(current, futureVelocity, c)
}

// ContextResonance pulls thought towards context along Poincaré geodesic.
func ContextResonance(thought, globalContext []float64, resonanceFactor float64, c float64) ([]float64, error) {
	pullDir, err := LogMap(thought, globalContext, c)
	if err != nil {
		return nil, err
	}
	factor := math.Max(0.0, math.Min(1.0, resonanceFactor))
	appliedPull := make([]float64, len(pullDir))
	for i, v := range pullDir {
		appliedPull[i] = v * factor
	}
	return ExpMap(thought, appliedPull, c)
}

// GenerateOrthogonalMatrix generates a deterministic orthogonal matrix from seedBytes.
func GenerateOrthogonalMatrix(dimension int, seedBytes []byte) [][]float64 {
	matrix := make([][]float64, dimension)
	for i := range matrix {
		matrix[i] = make([]float64, dimension)
	}

	hash := sha256.Sum256(seedBytes)
	currentHash := hash[:]
	hashOffset := 0

	for i := 0; i < dimension; i++ {
		for j := 0; j < dimension; j++ {
			if hashOffset >= 32 {
				nextHash := sha256.Sum256(currentHash)
				currentHash = nextHash[:]
				hashOffset = 0
			}
			valInt := binary.LittleEndian.Uint32(currentHash[hashOffset : hashOffset+4])
			val := (float64(valInt)/4294967296.0)*2.0 - 1.0
			matrix[i][j] = val
			hashOffset += 4
		}
	}

	// Gram-Schmidt Orthonormalization with Reorthogonalization
	for i := 0; i < dimension; i++ {
		v := make([]float64, dimension)
		copy(v, matrix[i])
		for j := 0; j < i; j++ {
			u := matrix[j]
			uDotV := dot(u, v)
			for k := 0; k < dimension; k++ {
				v[k] -= uDotV * u[k]
			}
			// Reorthogonalize
			uDotV = dot(u, v)
			for k := 0; k < dimension; k++ {
				v[k] -= uDotV * u[k]
			}
		}
		vNorm := norm(v)
		if vNorm > 1e-15 {
			for k := 0; k < dimension; k++ {
				matrix[i][k] = v[k] / vNorm
			}
		} else {
			for k := 0; k < dimension; k++ {
				matrix[i][k] = 0.0
			}
			matrix[i][i] = 1.0
		}
	}

	return matrix
}

// GenerateLorentzMatrix generates a deterministic Lorentz boost matrix from seedBytes.
func GenerateLorentzMatrix(dimension int, seedBytes []byte) [][]float64 {
	d := dimension - 1

	// R Matrix
	rSeedHash := sha256.New()
	rSeedHash.Write(seedBytes)
	rSeedHash.Write([]byte("spatial"))
	rSeed := rSeedHash.Sum(nil)
	R := GenerateOrthogonalMatrix(d, rSeed)

	// Boost vector beta
	bSeedHash := sha256.New()
	bSeedHash.Write(seedBytes)
	bSeedHash.Write([]byte("boost"))
	bSeed := bSeedHash.Sum(nil)

	beta := make([]float64, d)
	currentHash := bSeed
	hashOffset := 0
	for i := 0; i < d; i++ {
		if hashOffset >= 32 {
			nextHash := sha256.Sum256(currentHash)
			currentHash = nextHash[:]
			hashOffset = 0
		}
		valInt := binary.LittleEndian.Uint32(currentHash[hashOffset : hashOffset+4])
		val := (float64(valInt)/4294967296.0)*2.0 - 1.0
		beta[i] = val
		hashOffset += 4
	}

	betaNorm := norm(beta)
	if betaNorm > 1e-15 {
		for i := 0; i < d; i++ {
			beta[i] = (beta[i] / betaNorm) * 0.1
		}
	} else {
		for i := 0; i < d; i++ {
			beta[i] = 0.0
		}
		beta[0] = 0.1
	}

	betaSq := dot(beta, beta)
	gamma := 1.0 / math.Sqrt(1.0-betaSq)

	Lambda_B := make([][]float64, dimension)
	for i := range Lambda_B {
		Lambda_B[i] = make([]float64, dimension)
	}

	Lambda_B[0][0] = gamma
	for j := 1; j < dimension; j++ {
		Lambda_B[0][j] = -gamma * beta[j-1]
		Lambda_B[j][0] = -gamma * beta[j-1]
	}

	factor := (gamma - 1.0) / betaSq
	for i := 1; i < dimension; i++ {
		for j := 1; j < dimension; j++ {
			delta := 0.0
			if i == j {
				delta = 1.0
			}
			Lambda_B[i][j] = delta + factor*beta[i-1]*beta[j-1]
		}
	}

	Lambda_R := make([][]float64, dimension)
	for i := range Lambda_R {
		Lambda_R[i] = make([]float64, dimension)
	}
	Lambda_R[0][0] = 1.0
	for i := 1; i < dimension; i++ {
		for j := 1; j < dimension; j++ {
			Lambda_R[i][j] = R[i-1][j-1]
		}
	}

	// Multiply Lambda = Lambda_B * Lambda_R
	Lambda := make([][]float64, dimension)
	for i := range Lambda {
		Lambda[i] = make([]float64, dimension)
		for j := 0; j < dimension; j++ {
			sum := 0.0
			for k := 0; k < dimension; k++ {
				sum += Lambda_B[i][k] * Lambda_R[k][j]
			}
			Lambda[i][j] = sum
		}
	}

	return Lambda
}

// ProjectVector projects a vector using the given matrix.
func ProjectVector(v []float64, matrix [][]float64) []float64 {
	dim := len(v)
	projected := make([]float64, dim)
	for j := 0; j < dim; j++ {
		sum := 0.0
		for i := 0; i < dim; i++ {
			sum += v[i] * matrix[i][j]
		}
		projected[j] = sum
	}
	return projected
}

// InjectAnisotropicNoise injects deterministic anisotropic noise to prevent reconstruction.
func InjectAnisotropicNoise(v []float64, seedBytes []byte, sigma float64) []float64 {
	if len(v) == 0 || sigma <= 0.0 {
		return v
	}
	dim := len(v)
	noise := make([]float64, dim)
	hash := sha256.Sum256(seedBytes)
	currentHash := hash[:]
	hashOffset := 0

	for i := 0; i < dim; i++ {
		if hashOffset >= 32 {
			nextHash := sha256.Sum256(currentHash)
			currentHash = nextHash[:]
			hashOffset = 0
		}
		valInt := binary.LittleEndian.Uint32(currentHash[hashOffset : hashOffset+4])
		val := (float64(valInt)/4294967296.0)*2.0 - 1.0
		noise[i] = val
		hashOffset += 4
	}

	// Gram-Schmidt projection of noise to orthogonal direction
	vNorm := norm(v)
	if vNorm > 1e-15 {
		projLength := dot(noise, v) / (vNorm * vNorm)
		for i := 0; i < dim; i++ {
			noise[i] -= projLength * v[i]
		}
	}

	noiseNorm := norm(noise)
	res := make([]float64, dim)
	if noiseNorm > 1e-15 {
		scale := sigma * vNorm / noiseNorm
		for i := 0; i < dim; i++ {
			res[i] = v[i] + noise[i]*scale
		}
	} else {
		copy(res, v)
	}

	return res
}
