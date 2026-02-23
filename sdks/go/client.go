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
