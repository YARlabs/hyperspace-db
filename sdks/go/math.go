package hyperspace

import (
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
