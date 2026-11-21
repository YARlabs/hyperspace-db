package hyperspace

import (
	"context"
	"testing"

	"google.golang.org/grpc/metadata"

	pb "github.com/yarlabs/hyperspace-sdk-go/proto"
)

func TestBuildSearchRequestEfSearch(t *testing.T) {
	c := &HyperspaceClient{}
	req := c.buildSearchRequest([]float64{0.1, 0.2}, 5, "c", nil, &SearchParams{EfSearch: 300})
	if req.EfSearch == nil || *req.EfSearch != 300 {
		t.Fatalf("EfSearch not forwarded: %v", req.EfSearch)
	}
	req = c.buildSearchRequest([]float64{0.1, 0.2}, 5, "c", nil, &SearchParams{})
	if req.EfSearch != nil {
		t.Fatalf("EfSearch must stay unset (collection default), got %d", *req.EfSearch)
	}
	if req = c.buildSearchRequest([]float64{0.1}, 1, "c", nil, nil); req.EfSearch != nil {
		t.Fatal("nil params must not set EfSearch")
	}
}

func TestBuildSearchRequestCompactVectors(t *testing.T) {
	c := &HyperspaceClient{}
	c.SetCompactVectors(true)
	req := c.buildSearchRequest([]float64{0.5, 0.25}, 1, "c", nil, nil)
	if len(req.Vector) != 0 || len(req.VectorF32) != 2 {
		t.Fatalf("compact vectors: vector=%v vector_f32=%v", req.Vector, req.VectorF32)
	}
}

func TestBuildMultiCollectionRequestOptions(t *testing.T) {
	c := &HyperspaceClient{}
	req := c.buildMultiCollectionRequest([]string{"a", "b"}, []float64{0.1}, 7, WithEfSearch(64))
	if req.GetEfSearch() != 64 || req.GetTopK() != 7 {
		t.Fatalf("ef=%d topK=%d", req.GetEfSearch(), req.GetTopK())
	}
	if req = c.buildMultiCollectionRequest([]string{"a"}, []float64{0.1}, 3); req.EfSearch != nil {
		t.Fatal("no option = unset EfSearch")
	}
	var _ *pb.SearchMultiCollectionRequest = req
}

func TestWithContextKeepsCallerMetadata(t *testing.T) {
	c := &HyperspaceClient{apiKey: "k"}
	ctx := metadata.AppendToOutgoingContext(context.Background(), "x-expected-vectors", "5000000")
	md, _ := metadata.FromOutgoingContext(c.withContext(ctx))
	if got := md.Get("x-expected-vectors"); len(got) != 1 || got[0] != "5000000" {
		t.Fatalf("caller metadata dropped: %v", md)
	}
	if got := md.Get("x-api-key"); len(got) != 1 || got[0] != "k" {
		t.Fatalf("api key missing: %v", md)
	}
}
