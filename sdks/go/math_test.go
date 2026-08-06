package hyperspace

import (
	"context"
	"math"
	"testing"

	pb "github.com/yarlabs/hyperspace-sdk-go/proto"
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

func TestCrypto(t *testing.T) {
	// 1. Test Orthogonal matrix generation and distance preservation
	dim := 64
	seed := []byte("secret_seed_go")
	matrix := GenerateOrthogonalMatrix(dim, seed)

	// Orthogonality check: row i dot row j = delta_ij
	for i := 0; i < dim; i++ {
		for j := 0; j < dim; j++ {
			sum := 0.0
			for k := 0; k < dim; k++ {
				sum += matrix[i][k] * matrix[j][k]
			}
			expected := 0.0
			if i == j {
				expected = 1.0
			}
			if math.Abs(sum-expected) > 1e-9 {
				t.Errorf("Orthonormality failed at row %d, %d: got %f", i, j, sum)
			}
		}
	}

	// Distance preservation
	u := make([]float64, dim)
	v := make([]float64, dim)
	for i := 0; i < dim; i++ {
		u[i] = float64(i) * 0.1
		v[i] = float64(dim-i) * 0.15
	}

	uProj := ProjectVector(u, matrix)
	vProj := ProjectVector(v, matrix)

	distOrig := 0.0
	distProj := 0.0
	for i := 0; i < dim; i++ {
		distOrig += (u[i] - v[i]) * (u[i] - v[i])
		distProj += (uProj[i] - vProj[i]) * (uProj[i] - vProj[i])
	}
	distOrig = math.Sqrt(distOrig)
	distProj = math.Sqrt(distProj)

	if math.Abs(distOrig-distProj) > 1e-9 {
		t.Errorf("Distance preservation failed: orig=%f, proj=%f", distOrig, distProj)
	}

	// 2. Test Poincaré projection distance preservation
	poDim := 32
	poSeed := []byte("poincare_seed_go")
	poMatrix := GenerateLorentzMatrix(poDim+1, poSeed)

	pU := make([]float64, poDim)
	pV := make([]float64, poDim)
	for i := 0; i < poDim; i++ {
		pU[i] = float64(i) * 0.005
		pV[i] = float64(poDim-i) * 0.005
	}

	uLorentz := PoincareToLorentz(pU)
	vLorentz := PoincareToLorentz(pV)

	uLorentzProj := ProjectVector(uLorentz, poMatrix)
	vLorentzProj := ProjectVector(vLorentz, poMatrix)

	uProjPo := LorentzToPoincare(uLorentzProj)
	vProjPo := LorentzToPoincare(vLorentzProj)

	poincareDist := func(x, y []float64) float64 {
		xSq := 0.0
		ySq := 0.0
		diffSq := 0.0
		for i := range x {
			xSq += x[i] * x[i]
			ySq += y[i] * y[i]
			diffSq += (x[i] - y[i]) * (x[i] - y[i])
		}
		val := 1.0 + 2.0*diffSq/((1.0-xSq)*(1.0-ySq))
		return math.Acosh(val)
	}

	dOrig := poincareDist(pU, pV)
	dProj := poincareDist(uProjPo, vProjPo)

	if math.Abs(dOrig-dProj) > 1e-9 {
		t.Errorf("Poincare distance preservation failed: orig=%f, proj=%f", dOrig, dProj)
	}

	// 3. Test MRL Block-diagonal projection
	client := &HyperspaceClient{
		collectionKeys:        make(map[string]string),
		encryptionContexts:    make(map[string]*EncryptionContext),
		collectionMetrics:     make(map[string]string),
		collectionNoiseSigmas: make(map[string]float64),
		collectionSchemas:     make(map[string]*pb.CollectionSchema),
	}

	schema := &pb.CollectionSchema{
		Components: []*pb.VectorComponent{
			{Name: "primary", Metric: "cosine", FullDimension: 128, Weight: 1.0},
		},
		CascadePipeline: []*pb.MrlLayer{
			{ComponentName: "primary", CutoffDimension: 32, StoreInRam: true, RerankTopK: 100},
		},
	}
	client.RegisterCollectionKey("mrl_go", "secret_go", "cosine", 0.0, schema)

	context, err := client.getEncryptionContext(context.Background(), "mrl_go", 128, "cosine")
	if err != nil || context == nil {
		t.Fatalf("Failed to get encryption context: %v", err)
	}

	vec1 := make([]float64, 128)
	vec2 := make([]float64, 128)
	for i := 0; i < 128; i++ {
		vec1[i] = float64(i) * 0.05
		vec2[i] = float64(i) * 0.05
	}
	// Mutate tail
	for i := 32; i < 128; i++ {
		vec2[i] = float64(i) * 0.99
	}

	proj1 := client.projectCollectionVector("mrl_go", vec1, context, "cosine")
	proj2 := client.projectCollectionVector("mrl_go", vec2, context, "cosine")

	// Verify head is identical
	for i := 0; i < 32; i++ {
		if math.Abs(proj1[i]-proj2[i]) > 1e-12 {
			t.Errorf("MRL head mismatch at index %d: %f vs %f", i, proj1[i], proj2[i])
		}
	}

	// Verify tail is different
	tailDiff := false
	for i := 32; i < 128; i++ {
		if math.Abs(proj1[i]-proj2[i]) > 1e-5 {
			tailDiff = true
			break
		}
	}
	if !tailDiff {
		t.Error("Expected MRL tail dimensions to differ but they were identical")
	}
}
