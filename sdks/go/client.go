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

// CreateCollection initializes a new vector space
func (c *HyperspaceClient) CreateCollection(ctx context.Context, name string, dimension uint32, metric string) error {
	req := &pb.CreateCollectionRequest{
		Name:      name,
		Dimension: dimension,
		Metric:    metric,
	}

	_, err := c.client.CreateCollection(c.withContext(ctx), req)
	return err
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

// Search performs ANN lookup
func (c *HyperspaceClient) Search(ctx context.Context, vector []float64, topK uint32, collection string) ([]*pb.SearchResult, error) {
	req := &pb.SearchRequest{
		Vector:     vector,
		TopK:       topK,
		Collection: collection,
	}

	res, err := c.client.Search(c.withContext(ctx), req)
	if err != nil {
		return nil, err
	}

	return res.Results, nil
}

// SearchText performs ANN lookup using text input (vectorized on server)
func (c *HyperspaceClient) SearchText(ctx context.Context, text string, topK uint32, collection string) ([]*pb.SearchResult, error) {
	req := &pb.SearchTextRequest{
		Text:       text,
		TopK:       topK,
		Collection: collection,
	}
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
