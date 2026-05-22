package hyperspace

import (
	"math"
	"testing"
)

func TestMobiusAddIdentity(t *testing.T) {
	x := []float64{0.1, -0.2, 0.05}
	zero := []float64{0.0, 0.0, 0.0}
	out, err := MobiusAdd(x, zero, 1.0)
	if err != nil {
		t.Fatalf("MobiusAdd failed: %v", err)
	}
	for i := range x {
		if math.Abs(out[i]-x[i]) > 1e-12 {
			t.Errorf("Expected %f, got %f at index %d", x[i], out[i], i)
		}
	}
}

func TestExpLogRoundtripSmallStep(t *testing.T) {
	x := []float64{0.05, -0.03}
	v := []float64{0.001, 0.002}
	y, err := ExpMap(x, v, 1.0)
	if err != nil {
		t.Fatalf("ExpMap failed: %v", err)
	}
	vBack, err := LogMap(x, y, 1.0)
	if err != nil {
		t.Fatalf("LogMap failed: %v", err)
	}
	for i := range v {
		if math.Abs(v[i]-vBack[i]) > 1e-6 {
			t.Errorf("Expected %f, got %f at index %d", v[i], vBack[i], i)
		}
	}
}

func TestCognitiveMath(t *testing.T) {
	// Local Entropy
	candidate := []float64{0.1, 0.1}
	neighbors := [][]float64{
		{0.11, 0.1},
		{0.1, 0.12},
		{0.09, 0.09},
	}
	entropy, err := LocalEntropy(candidate, neighbors, 1.0)
	if err != nil {
		t.Fatalf("LocalEntropy failed: %v", err)
	}
	if entropy >= 0.1 {
		t.Errorf("Expected entropy < 0.1, got %f", entropy)
	}

	// Lyapunov Convergence
	trajectoryConverging := [][]float64{
		{0.5, 0.5},
		{0.3, 0.3},
		{0.1, 0.1},
		{0.05, 0.05},
	}
	lyapunov, err := LyapunovConvergence(trajectoryConverging, 1.0)
	if err != nil {
		t.Fatalf("LyapunovConvergence failed: %v", err)
	}
	if lyapunov >= 0.0 {
		t.Errorf("Expected Lyapunov < 0.0, got %f", lyapunov)
	}

	// Context Resonance
	thought := []float64{0.5, 0.0}
	globalCtx := []float64{0.0, 0.5}
	pull, err := ContextResonance(thought, globalCtx, 0.5, 1.0)
	if err != nil {
		t.Fatalf("ContextResonance failed: %v", err)
	}
	if len(pull) != 2 {
		t.Errorf("Expected resonance output length 2, got %d", len(pull))
	}

	// Koopman Extrapolation
	past := []float64{0.1, 0.2}
	current := []float64{0.15, 0.25}
	predicted, err := KoopmanExtrapolate(past, current, 1.0, 1.0)
	if err != nil {
		t.Fatalf("KoopmanExtrapolate failed: %v", err)
	}
	if len(predicted) != 2 {
		t.Errorf("Expected extrapolated output length 2, got %d", len(predicted))
	}
}

func TestLorentzProductAndDistance(t *testing.T) {
	u := []float64{2.0, math.Sqrt(3.0)} // ||u||^2 = -4 + 3 = -1
	v := []float64{2.0, math.Sqrt(3.0)}
	dist := LorentzDist(u, v)
	if dist >= 1e-4 {
		t.Errorf("Expected LorentzDist close to 0, got %f", dist)
	}

	w := []float64{3.0, math.Sqrt(8.0)} // ||w||^2 = -9 + 8 = -1
	product := LorentzProduct(u, w)
	expected := -2.0*3.0 + math.Sqrt(3.0)*math.Sqrt(8.0)
	if math.Abs(product-expected) > 1e-4 {
		t.Errorf("Expected product %f, got %f", expected, product)
	}
}
