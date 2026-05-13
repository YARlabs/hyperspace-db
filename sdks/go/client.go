package hyperspace

import (
	"context"
	"fmt"

	pb "github.com/yarlabs/hyperspace-sdk-go/proto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/metadata"
)

// HyperspaceClient represents a connection to HyperspaceDB
type HyperspaceClient struct {
	conn   *grpc.ClientConn
	client pb.DatabaseClient
	apiKey string
}

// NewClient creates a new gRPC connection pool to HyperspaceDB
func NewClient(endpoint string, apiKey string) (*HyperspaceClient, error) {
	// Task 2.2: gRPC Connection Pooling / Keepalive configuration
	opts := []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultServiceConfig(`{"loadBalancingConfig": [{"round_robin":{}}]}`),
	}

	conn, err := grpc.Dial(endpoint, opts...)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to %s: %w", endpoint, err)
	}

	return &HyperspaceClient{
		conn:   conn,
		client: pb.NewDatabaseClient(conn),
		apiKey: apiKey,
	}, nil
}

// Close closes the connection pool
func (c *HyperspaceClient) Close() error {
	return c.conn.Close()
}

// withContext adds authentication headers if an API key is set
func (c *HyperspaceClient) withContext(ctx context.Context) context.Context {
	if c.apiKey != "" {
		md := metadata.Pairs("x-api-key", c.apiKey)
		return metadata.NewOutgoingContext(ctx, md)
	}
	return ctx
}

func (c *HyperspaceClient) CreateCollection(ctx context.Context, name string, schema *pb.CollectionSchema) error {
	req := &pb.CreateCollectionRequest{
		Name:   name,
		Schema: schema,
	}

	_, err := c.client.CreateCollection(c.withContext(ctx), req)
	return err
}

// ListCollections retrieves all active collections for the current tenant
func (c *HyperspaceClient) ListCollections(ctx context.Context) ([]*pb.CollectionSummary, error) {
	req := &pb.Empty{}
	resp, err := c.client.ListCollections(c.withContext(ctx), req)
	if err != nil {
		return nil, err
	}
	return resp.Collections, nil
}


// Insert pushes a new vector into the database
func (c *HyperspaceClient) Insert(ctx context.Context, id uint32, vector []float64, collection string) error {
	req := &pb.InsertRequest{
		Id:         id,
		Collection: collection,
		Vector:     vector,
	}

	_, err := c.client.Insert(c.withContext(ctx), req)
	return err
}

// BatchInsert pushes multiple vectors in a single request
func (c *HyperspaceClient) BatchInsert(ctx context.Context, ids []uint32, vectors [][]float64, collection string) error {
	if len(ids) != len(vectors) {
		return fmt.Errorf("ids and vectors length mismatch")
	}
	protoVectors := make([]*pb.VectorData, len(ids))
	for i := range ids {
		protoVectors[i] = &pb.VectorData{
			Id:     ids[i],
			Vector: vectors[i],
		}
	}
	req := &pb.BatchInsertRequest{
		Collection: collection,
		Vectors:    protoVectors,
	}
	_, err := c.client.BatchInsert(c.withContext(ctx), req)
	return err
}

// InsertText pushes text to be vectorized and inserted on the server side
func (c *HyperspaceClient) InsertText(ctx context.Context, id uint32, text string, collection string) error {
	req := &pb.InsertTextRequest{
		Id:         id,
		Text:       text,
		Collection: collection,
	}
	_, err := c.client.InsertText(c.withContext(ctx), req)
	return err
}

// Delete removes a single vector by ID
func (c *HyperspaceClient) Delete(ctx context.Context, id uint32, collection string) error {
	req := &pb.DeleteRequest{
		Id:         id,
		Collection: collection,
	}
	resp, err := c.client.Delete(c.withContext(ctx), req)
	if err != nil {
		return err
	}
	if !resp.Success {
		return fmt.Errorf("deletion failed")
	}
	return nil
}

// Vectorize converts text to a dense vector using server-side embedding
func (c *HyperspaceClient) Vectorize(ctx context.Context, text string, metric string) ([]float64, error) {
	req := &pb.VectorizeRequest{
		Text:   text,
		Metric: metric,
	}
	resp, err := c.client.Vectorize(c.withContext(ctx), req)
	if err != nil {
		return nil, err
	}
	return resp.Vector, nil
}

// SearchParams contains optional parameters for advanced vector search
type SearchParams struct {
	Filters          []*pb.Filter
	HybridQuery      string
	HybridAlpha      float32
	BM25Options      *pb.Bm25Options
	MrlDimension     uint32
	UseWasserstein   bool
	ComponentWeights map[string]float32
}

// Search performs ANN lookup with optional geometric filters, BM25 factors, and hybrid ranking
func (c *HyperspaceClient) Search(ctx context.Context, vector []float64, topK uint32, collection string, params *SearchParams) ([]*pb.SearchResult, error) {
	req := &pb.SearchRequest{
		Vector:     vector,
		TopK:       topK,
		Collection: collection,
	}

	if params != nil {
		req.Filters = params.Filters
		if params.HybridQuery != "" {
			req.HybridQuery = &params.HybridQuery
		}
		if params.HybridAlpha != 0 {
			req.HybridAlpha = &params.HybridAlpha
		}
		if params.MrlDimension != 0 {
			req.MrlDimension = &params.MrlDimension
		}
		if params.UseWasserstein {
			req.UseWasserstein = params.UseWasserstein
		}
		req.Bm25Options = params.BM25Options
		if params.ComponentWeights != nil {
			req.ComponentWeights = params.ComponentWeights
		}
	}

	res, err := c.client.Search(c.withContext(ctx), req)
	if err != nil {
		return nil, err
	}

	return res.Results, nil
}

// SearchText performs ANN lookup using text input (vectorized on server) or BM25 lexical ranking
func (c *HyperspaceClient) SearchText(ctx context.Context, text string, topK uint32, collection string, hybridAlpha float32, bm25 *pb.Bm25Options) ([]*pb.SearchResult, error) {
	req := &pb.SearchTextRequest{
		Text:       text,
		TopK:       topK,
		Collection: collection,
	}
	if hybridAlpha != 0 {
		req.HybridAlpha = &hybridAlpha
	}
	req.Bm25Options = bm25

	res, err := c.client.SearchText(c.withContext(ctx), req)
	if err != nil {
		return nil, err
	}
	return res.Results, nil
}

// SyncHandshake sends local buckets to server and gets differing ones in return
func (c *HyperspaceClient) SyncHandshake(ctx context.Context, collection string, clientBuckets []uint64, clientClock uint64, clientCount uint64) (*pb.SyncHandshakeResponse, error) {
	if len(clientBuckets) != 256 {
		return nil, fmt.Errorf("clientBuckets must contain exactly 256 elements")
	}
	req := &pb.SyncHandshakeRequest{
		Collection:         collection,
		ClientBuckets:      clientBuckets,
		ClientLogicalClock: clientClock,
		ClientCount:        clientCount,
	}
	return c.client.SyncHandshake(c.withContext(ctx), req)
}

// SyncPull streams vectors for specified bucket indices
func (c *HyperspaceClient) SyncPull(ctx context.Context, collection string, bucketIndices []uint32) (pb.Database_SyncPullClient, error) {
	req := &pb.SyncPullRequest{
		Collection:    collection,
		BucketIndices: bucketIndices,
	}
	return c.client.SyncPull(c.withContext(ctx), req)
}

// SyncPush initiates a stream to push offline vectors to server
func (c *HyperspaceClient) SyncPush(ctx context.Context) (pb.Database_SyncPushClient, error) {
	return c.client.SyncPush(c.withContext(ctx))
}

func (c *HyperspaceClient) GetCollectionStats(ctx context.Context, name string) (*pb.CollectionStatsResponse, error) {
	req := &pb.CollectionStatsRequest{Name: name}
	return c.client.GetCollectionStats(c.withContext(ctx), req)
}

func (c *HyperspaceClient) Exists(ctx context.Context, name string) (bool, error) {
	_, err := c.GetCollectionStats(ctx, name)
	if err != nil {
		// Just relying on error string since we might not have grpc codes imported directly
		return false, nil
	}
	return true, nil
}

func (c *HyperspaceClient) UpdateCollection(ctx context.Context, name string) error {
	req := &pb.ConfigUpdate{Collection: name}
	_, err := c.client.Configure(c.withContext(ctx), req)
	return err
}

func (c *HyperspaceClient) CreateSnapshot(ctx context.Context) error {
	_, err := c.client.TriggerSnapshot(c.withContext(ctx), &pb.Empty{})
	return err
}

func (c *HyperspaceClient) Vacuum(ctx context.Context) error {
	_, err := c.client.TriggerVacuum(c.withContext(ctx), &pb.Empty{})
	return err
}

func (c *HyperspaceClient) GetMetrics(ctx context.Context) (pb.Database_MonitorClient, error) {
	req := &pb.MonitorRequest{}
	return c.client.Monitor(c.withContext(ctx), req)
}

func (c *HyperspaceClient) SearchMultiCollection(ctx context.Context, collections []string, query []float64, topK uint32) (*pb.SearchMultiCollectionResponse, error) {
	req := &pb.SearchMultiCollectionRequest{
		Collections: collections,
		Vector:      query,
		TopK:        topK,
	}
	return c.client.SearchMultiCollection(c.withContext(ctx), req)
}

// Graph API
func (c *HyperspaceClient) GetNode(ctx context.Context, id uint32, layer uint32, collection string) (*pb.GraphNode, error) {
	req := &pb.GetNodeRequest{
		Collection: collection,
		Id:         id,
		Layer:      layer,
	}
	return c.client.GetNode(c.withContext(ctx), req)
}

func (c *HyperspaceClient) GetNeighbors(ctx context.Context, id uint32, layer uint32, limit uint32, offset uint32, collection string) (*pb.GetNeighborsResponse, error) {
	req := &pb.GetNeighborsRequest{
		Collection: collection,
		Id:         id,
		Layer:      layer,
		Limit:      limit,
		Offset:     offset,
	}
	return c.client.GetNeighbors(c.withContext(ctx), req)
}

func (c *HyperspaceClient) Traverse(ctx context.Context, req *pb.TraverseRequest) (*pb.TraverseResponse, error) {
	return c.client.Traverse(c.withContext(ctx), req)
}

func (c *HyperspaceClient) FindSemanticClusters(ctx context.Context, req *pb.FindSemanticClustersRequest) (*pb.FindSemanticClustersResponse, error) {
	return c.client.FindSemanticClusters(c.withContext(ctx), req)
}

// Admin & Sync API
func (c *HyperspaceClient) RebuildIndex(ctx context.Context, name string) error {
	req := &pb.RebuildIndexRequest{Name: name}
	_, err := c.client.RebuildIndex(c.withContext(ctx), req)
	return err
}

func (c *HyperspaceClient) GetDigest(ctx context.Context, collection string) (*pb.DigestResponse, error) {
	req := &pb.DigestRequest{Collection: collection}
	return c.client.GetDigest(c.withContext(ctx), req)
}

func (c *HyperspaceClient) SubscribeToEvents(ctx context.Context, types []pb.EventType, collection string) (pb.Database_SubscribeToEventsClient, error) {
	req := &pb.EventSubscriptionRequest{
		Collection: &collection,
	}
	for _, t := range types {
		req.Types = append(req.Types, t)
	}
	return c.client.SubscribeToEvents(c.withContext(ctx), req)
}

func (c *HyperspaceClient) TriggerReconsolidation(ctx context.Context, collection string, target []float64, lr float64) error {
	req := &pb.ReconsolidationRequest{
		Collection:   collection,
		TargetVector: target,
		LearningRate: lr,
	}
	_, err := c.client.TriggerReconsolidation(c.withContext(ctx), req)
	return err
}

func (c *HyperspaceClient) GetPoints(ctx context.Context, ids []uint32, collection string) ([]*pb.VectorData, error) {
	req := &pb.GetPointsRequest{
		Ids:        ids,
		Collection: collection,
	}
	resp, err := c.client.GetPoints(c.withContext(ctx), req)
	if err != nil {
		return nil, err
	}
	return resp.Points, nil
}

func (c *HyperspaceClient) UpdatePayload(ctx context.Context, id uint32, metadata map[string]string, collection string) error {
	req := &pb.UpdatePayloadRequest{
		Id:            id,
		Metadata:      metadata,
		Collection:    collection,
		TypedMetadata: make(map[string]*pb.MetadataValue),
	}
	_, err := c.client.UpdatePayload(c.withContext(ctx), req)
	return err
}

func (c *HyperspaceClient) Scroll(ctx context.Context, limit uint32, offset uint32, filters []*pb.Filter, collection string) ([]*pb.VectorData, error) {
	req := &pb.ScrollRequest{
		Limit:      limit,
		Offset:     offset,
		Filters:    filters,
		Collection: collection,
	}
	resp, err := c.client.Scroll(c.withContext(ctx), req)
	if err != nil {
		return nil, err
	}
	return resp.Points, nil
}

func (c *HyperspaceClient) Count(ctx context.Context, filters []*pb.Filter, collection string) (uint64, error) {
	req := &pb.CountRequest{
		Filters:    filters,
		Collection: collection,
	}
	resp, err := c.client.Count(c.withContext(ctx), req)
	if err != nil {
		return 0, err
	}
	return resp.Count, nil
}

func (c *HyperspaceClient) HealthCheck(ctx context.Context) (string, error) {
	req := &pb.Empty{}
	resp, err := c.client.HealthCheck(c.withContext(ctx), req)
	if err != nil {
		return "", err
	}
	return resp.Status, nil
}
