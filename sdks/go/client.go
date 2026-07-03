package hyperspace

import (
	"context"
	"crypto/aes"
	"crypto/cipher"
	"crypto/hmac"
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"strings"

	pb "github.com/yarlabs/hyperspace-sdk-go/proto"
	"golang.org/x/crypto/pbkdf2"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/metadata"
)

type EncryptionContext struct {
	aesKey             []byte
	hmacKey            []byte
	projectionMatrices map[string][][]float64
}

// HyperspaceClient represents a connection to HyperspaceDB
type HyperspaceClient struct {
	conn                  *grpc.ClientConn
	client                pb.DatabaseClient
	apiKey                string
	collectionKeys        map[string]string
	encryptionContexts    map[string]*EncryptionContext
	collectionMetrics     map[string]string
	collectionNoiseSigmas map[string]float64
	collectionSchemas     map[string]*pb.CollectionSchema
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
		conn:                  conn,
		client:                pb.NewDatabaseClient(conn),
		apiKey:                apiKey,
		collectionKeys:        make(map[string]string),
		encryptionContexts:    make(map[string]*EncryptionContext),
		collectionMetrics:     make(map[string]string),
		collectionNoiseSigmas: make(map[string]float64),
		collectionSchemas:     make(map[string]*pb.CollectionSchema),
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

func (c *HyperspaceClient) CreateCollection(ctx context.Context, name string, schema *pb.CollectionSchema, encryptionKey string, noiseSigma float64) error {
	metric := "l2"
	if len(schema.Components) > 0 {
		metric = schema.Components[0].Metric
	}
	if encryptionKey != "" {
		c.RegisterCollectionKey(name, encryptionKey, metric, noiseSigma, schema)
	}
	req := &pb.CreateCollectionRequest{
		Name:   name,
		Schema: schema,
	}

	_, err := c.client.CreateCollection(c.withContext(ctx), req)
	return err
}

func (c *HyperspaceClient) FreezeCollection(ctx context.Context, name string) (string, error) {
	req := &pb.FreezeCollectionRequest{
		Name: name,
	}
	resp, err := c.client.FreezeCollection(c.withContext(ctx), req)
	if err != nil {
		return "", err
	}
	return resp.Status, nil
}

func (c *HyperspaceClient) UnfreezeCollection(ctx context.Context, name string) (string, error) {
	req := &pb.UnfreezeCollectionRequest{
		Name: name,
	}
	resp, err := c.client.UnfreezeCollection(c.withContext(ctx), req)
	if err != nil {
		return "", err
	}
	return resp.Status, nil
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


type InsertParams struct {
	Metadata      map[string]string
	TypedMetadata map[string]*pb.MetadataValue
	Payload       []byte
	Durability    pb.DurabilityLevel
}

// Insert pushes a new vector into the database (supports ZK client-side encryption)
func (c *HyperspaceClient) Insert(ctx context.Context, id uint32, vector []float64, collection string, params *InsertParams) error {
	metric := "l2"
	if c.collectionMetrics != nil {
		if m, ok := c.collectionMetrics[collection]; ok {
			metric = m
		}
	}
	context, err := c.getEncryptionContext(ctx, collection, len(vector), metric)
	if err != nil {
		return err
	}

	var finalVector []float64
	var finalMetadata map[string]string
	var finalTypedMetadata map[string]*pb.MetadataValue
	var finalPayload []byte

	if context != nil {
		// 1. Noise injection
		sigma := 0.02
		if s, ok := c.collectionNoiseSigmas[collection]; ok {
			sigma = s
		}
		if sigma > 0.0 {
			finalVector = InjectAnisotropicNoise(vector, context.hmacKey, sigma)
		} else {
			finalVector = make([]float64, len(vector))
			copy(finalVector, vector)
		}

		// 2. Vector projection
		finalVector = c.projectCollectionVector(collection, finalVector, context, metric)

		// 3. Payload Encryption
		if params != nil && len(params.Payload) > 0 {
			encPayload, err := c.encryptPayload(params.Payload, context.aesKey)
			if err != nil {
				return err
			}
			finalPayload = encPayload
		}

		// 4. Metadata Hashing
		if params != nil && len(params.Metadata) > 0 {
			finalMetadata = make(map[string]string)
			for k, v := range params.Metadata {
				ek := c.hashMetadataKey(k, context.hmacKey)
				ev := c.hashMetadataValue(v, context.hmacKey)
				finalMetadata[ek] = ev
			}
		}

		if params != nil && len(params.TypedMetadata) > 0 {
			finalTypedMetadata = make(map[string]*pb.MetadataValue)
			for k, v := range params.TypedMetadata {
				ek := c.hashMetadataKey(k, context.hmacKey)
				var ev string
				if v.GetStringValue() != "" {
					ev = v.GetStringValue()
				} else if v.GetIntValue() != 0 {
					ev = fmt.Sprintf("%d", v.GetIntValue())
				} else if v.GetBoolValue() {
					ev = "true"
				} else {
					ev = "false"
				}
				hashedEv := c.hashMetadataValue(ev, context.hmacKey)
				finalTypedMetadata[ek] = &pb.MetadataValue{
					Kind: &pb.MetadataValue_StringValue{StringValue: hashedEv},
				}
			}
		}
	} else {
		finalVector = vector
		if params != nil {
			finalMetadata = params.Metadata
			finalTypedMetadata = params.TypedMetadata
			finalPayload = params.Payload
		}
	}

	req := &pb.InsertRequest{
		Id:            id,
		Collection:    collection,
		Vector:        finalVector,
		Metadata:      finalMetadata,
		TypedMetadata: finalTypedMetadata,
		Payload:       finalPayload,
	}
	if params != nil {
		req.Durability = params.Durability
	}

	_, err = c.client.Insert(c.withContext(ctx), req)
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
	UseWave          bool
	RestartFactor    *float32
}

// Search performs ANN lookup with optional geometric filters, BM25 factors, and hybrid ranking
func (c *HyperspaceClient) Search(ctx context.Context, vector []float64, topK uint32, collection string, params *SearchParams) ([]*pb.SearchResult, error) {
	metric := "l2"
	if c.collectionMetrics != nil {
		if m, ok := c.collectionMetrics[collection]; ok {
			metric = m
		}
	}
	context, err := c.getEncryptionContext(ctx, collection, len(vector), metric)
	if err != nil {
		return nil, err
	}

	var finalVector []float64
	var finalFilters []*pb.Filter

	if context != nil {
		// 1. Noise injection
		sigma := 0.02
		if s, ok := c.collectionNoiseSigmas[collection]; ok {
			sigma = s
		}
		if sigma > 0.0 {
			finalVector = InjectAnisotropicNoise(vector, context.hmacKey, sigma)
		} else {
			finalVector = make([]float64, len(vector))
			copy(finalVector, vector)
		}

		// 2. Vector projection
		finalVector = c.projectCollectionVector(collection, finalVector, context, metric)

		// 3. Encrypt filters
		if params != nil {
			if len(params.Filters) > 0 {
				finalFilters = c.encryptFilters(params.Filters, context)
			}
		}
	} else {
		finalVector = vector
		if params != nil {
			finalFilters = params.Filters
		}
	}

	req := &pb.SearchRequest{
		Vector:     finalVector,
		TopK:       topK,
		Collection: collection,
	}

	if params != nil {
		req.Filters = finalFilters
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
		req.UseWave = params.UseWave
		if params.RestartFactor != nil {
			if req.Filter == nil {
				req.Filter = make(map[string]string)
			}
			req.Filter["wave_restart_factor"] = fmt.Sprintf("%g", *params.RestartFactor)
		}
	}

	res, err := c.client.Search(c.withContext(ctx), req)
	if err != nil {
		return nil, err
	}

	// Decrypt result payloads if context exists
	if context != nil {
		for _, r := range res.Results {
			if len(r.Payload) > 0 {
				decPayload, err := c.decryptPayload(r.Payload, context.aesKey)
				if err == nil {
					r.Payload = decPayload
				}
			}
		}
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

func (c *HyperspaceClient) GetSubsumptionTree(ctx context.Context, rootId uint32, maxDepth uint32, collection string) (*pb.GetSubsumptionTreeResponse, error) {
	req := &pb.GetSubsumptionTreeRequest{
		Collection: collection,
		RootId:     rootId,
		MaxDepth:   maxDepth,
	}
	return c.client.GetSubsumptionTree(c.withContext(ctx), req)
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

func (c *HyperspaceClient) RegisterCollectionKey(collectionName string, key string, metric string, noiseSigma float64, schema *pb.CollectionSchema) {
	c.collectionKeys[collectionName] = key
	c.collectionMetrics[collectionName] = metric
	c.collectionNoiseSigmas[collectionName] = noiseSigma
	if schema != nil {
		c.collectionSchemas[collectionName] = schema
	}
	delete(c.encryptionContexts, collectionName)
}

func (c *HyperspaceClient) deriveKeys(password string, collectionName string) ([]byte, []byte) {
	saltHash := sha256.Sum256([]byte(collectionName))
	salt := saltHash[:]
	aesKey := pbkdf2.Key([]byte(password), salt, 100000, 32, sha256.New)
	hmacKey := pbkdf2.Key([]byte(password), salt, 100000, 32, sha256.New)
	return aesKey, hmacKey
}

func (c *HyperspaceClient) encryptPayload(plaintext []byte, aesKey []byte) ([]byte, error) {
	pbkdf2Salt := make([]byte, 16)
	if _, err := io.ReadFull(rand.Reader, pbkdf2Salt); err != nil {
		return nil, err
	}

	derivedKey := pbkdf2.Key(aesKey, pbkdf2Salt, 100000, 32, sha256.New)

	block, err := aes.NewCipher(derivedKey)
	if err != nil {
		return nil, err
	}

	aesgcm, err := cipher.NewGCM(block)
	if err != nil {
		return nil, err
	}

	iv := make([]byte, 12)
	if _, err := io.ReadFull(rand.Reader, iv); err != nil {
		return nil, err
	}

	ciphertextAndTag := aesgcm.Seal(nil, iv, plaintext, nil)

	result := make([]byte, 0, 16+12+len(ciphertextAndTag))
	result = append(result, pbkdf2Salt...)
	result = append(result, iv...)
	result = append(result, ciphertextAndTag...)
	return result, nil
}

func (c *HyperspaceClient) decryptPayload(data []byte, aesKey []byte) ([]byte, error) {
	if len(data) < 16+12+16 {
		return nil, fmt.Errorf("invalid encrypted payload size")
	}

	pbkdf2Salt := data[:16]
	iv := data[16:28]
	ciphertextAndTag := data[28:]

	derivedKey := pbkdf2.Key(aesKey, pbkdf2Salt, 100000, 32, sha256.New)

	block, err := aes.NewCipher(derivedKey)
	if err != nil {
		return nil, err
	}

	aesgcm, err := cipher.NewGCM(block)
	if err != nil {
		return nil, err
	}

	return aesgcm.Open(nil, iv, ciphertextAndTag, nil)
}

func (c *HyperspaceClient) hashMetadataKey(key string, hmacKey []byte) string {
	mac := hmac.New(sha256.New, hmacKey)
	mac.Write([]byte(key))
	h := hex.EncodeToString(mac.Sum(nil))
	return "tag_" + h[:16]
}

func (c *HyperspaceClient) hashMetadataValue(value string, hmacKey []byte) string {
	mac := hmac.New(sha256.New, hmacKey)
	mac.Write([]byte(value))
	h := hex.EncodeToString(mac.Sum(nil))
	return "val_" + h
}

func (c *HyperspaceClient) getEncryptionContext(ctx context.Context, collection string, vectorDim int, metric string) (*EncryptionContext, error) {
	if collection == "" {
		return nil, nil
	}
	key, ok := c.collectionKeys[collection]
	if !ok {
		return nil, nil
	}

	if _, ok := c.collectionSchemas[collection]; !ok {
		stats, err := c.GetCollectionStats(ctx, collection)
		if err == nil && stats != nil && stats.Schema != nil {
			c.collectionSchemas[collection] = stats.Schema
		}
	}

	if _, ok := c.encryptionContexts[collection]; !ok {
		aesKey, hmacKey := c.deriveKeys(key, collection)
		c.encryptionContexts[collection] = &EncryptionContext{
			aesKey:             aesKey,
			hmacKey:            hmacKey,
			projectionMatrices: make(map[string][][]float64),
		}
	}

	context := c.encryptionContexts[collection]

	if vectorDim > 0 {
		cacheKey := fmt.Sprintf("%d", vectorDim)
		if _, ok := context.projectionMatrices[cacheKey]; !ok {
			isLorentz := strings.ToLower(metric) == "lorentz" || strings.ToLower(metric) == "poincare"
			matrixDim := vectorDim
			if strings.ToLower(metric) == "poincare" {
				matrixDim = vectorDim + 1
			}
			if isLorentz {
				context.projectionMatrices[cacheKey] = GenerateLorentzMatrix(matrixDim, context.hmacKey)
			} else {
				context.projectionMatrices[cacheKey] = GenerateOrthogonalMatrix(matrixDim, context.hmacKey)
			}
		}
	}

	return context, nil
}

func (c *HyperspaceClient) projectSingleBlock(subVec []float64, metric string, context *EncryptionContext, blockId string) []float64 {
	dim := len(subVec)
	if dim == 0 {
		return []float64{}
	}

	cacheKey := fmt.Sprintf("%d", dim)
	if blockId != "" {
		cacheKey = fmt.Sprintf("%d_%s", dim, blockId)
	}

	if _, ok := context.projectionMatrices[cacheKey]; !ok {
		isLorentz := strings.ToLower(metric) == "lorentz" || strings.ToLower(metric) == "poincare"
		matrixDim := dim
		if strings.ToLower(metric) == "poincare" {
			matrixDim = dim + 1
		}

		seed := context.hmacKey
		if blockId != "" {
			h := sha256.New()
			h.Write(seed)
			h.Write([]byte(blockId))
			seed = h.Sum(nil)
		}

		if isLorentz {
			context.projectionMatrices[cacheKey] = GenerateLorentzMatrix(matrixDim, seed)
		} else {
			context.projectionMatrices[cacheKey] = GenerateOrthogonalMatrix(matrixDim, seed)
		}
	}

	matrix := context.projectionMatrices[cacheKey]

	if strings.ToLower(metric) == "poincare" {
		lorentzVec := PoincareToLorentz(subVec)
		projLorentz := ProjectVector(lorentzVec, matrix)
		return LorentzToPoincare(projLorentz)
	} else {
		return ProjectVector(subVec, matrix)
	}
}

func (c *HyperspaceClient) projectCollectionVector(collection string, vector []float64, context *EncryptionContext, metric string) []float64 {
	schema, ok := c.collectionSchemas[collection]
	if !ok || schema == nil || len(schema.Components) == 0 {
		return c.projectSingleBlock(vector, metric, context, "")
	}

	componentCutoffs := make(map[string][]int)
	for _, layer := range schema.CascadePipeline {
		compName := layer.ComponentName
		cutoff := int(layer.CutoffDimension)
		if compName != "" && cutoff > 0 {
			componentCutoffs[compName] = append(componentCutoffs[compName], cutoff)
		}
	}

	for compName, cutoffs := range componentCutoffs {
		unique := make(map[int]bool)
		var sorted []int
		for _, c := range cutoffs {
			if !unique[c] {
				unique[c] = true
				sorted = append(sorted, c)
			}
		}
		for i := 0; i < len(sorted); i++ {
			for j := i + 1; j < len(sorted); j++ {
				if sorted[i] > sorted[j] {
					sorted[i], sorted[j] = sorted[j], sorted[i]
				}
			}
		}
		componentCutoffs[compName] = sorted
	}

	var projectedParts []float64
	currentOffset := 0

	for _, comp := range schema.Components {
		compName := comp.Name
		compMetric := comp.Metric
		compDim := int(comp.FullDimension)

		if currentOffset >= len(vector) {
			break
		}

		end := currentOffset + compDim
		if end > len(vector) {
			end = len(vector)
		}
		subVec := make([]float64, compDim)
		copy(subVec, vector[currentOffset:end])

		cutoffs := componentCutoffs[compName]
		var validCutoffs []int
		for _, c := range cutoffs {
			if c < compDim {
				validCutoffs = append(validCutoffs, c)
			}
		}

		var projSub []float64
		if len(validCutoffs) == 0 {
			projSub = c.projectSingleBlock(subVec, compMetric, context, "")
		} else {
			blockStart := 0
			for _, cutoff := range validCutoffs {
				blockData := subVec[blockStart:cutoff]
				blockId := fmt.Sprintf("%s_block_%d_%d", compName, blockStart, cutoff)
				projBlock := c.projectSingleBlock(blockData, compMetric, context, blockId)
				projSub = append(projSub, projBlock...)
				blockStart = cutoff
			}
			if blockStart < compDim {
				blockData := subVec[blockStart:compDim]
				blockId := fmt.Sprintf("%s_block_%d_%d", compName, blockStart, compDim)
				projBlock := c.projectSingleBlock(blockData, compMetric, context, blockId)
				projSub = append(projSub, projBlock...)
			}
		}

		projectedParts = append(projectedParts, projSub...)
		currentOffset += compDim
	}

	if currentOffset < len(vector) {
		projectedParts = append(projectedParts, vector[currentOffset:]...)
	}

	return projectedParts
}

func (c *HyperspaceClient) encryptFilters(filters []*pb.Filter, context *EncryptionContext) []*pb.Filter {
	if filters == nil {
		return nil
	}
	res := make([]*pb.Filter, len(filters))
	for i, f := range filters {
		nf := &pb.Filter{}
		if f.GetMatch() != nil {
			ek := c.hashMetadataKey(f.GetMatch().Key, context.hmacKey)
			ev := c.hashMetadataValue(f.GetMatch().Value, context.hmacKey)
			nf.Condition = &pb.Filter_Match{
				Match: &pb.Match{
					Key:   ek,
					Value: ev,
				},
			}
		} else if f.GetPrefix() != nil {
			ek := c.hashMetadataKey(f.GetPrefix().Key, context.hmacKey)
			ev := c.hashMetadataValue(f.GetPrefix().Prefix, context.hmacKey)
			nf.Condition = &pb.Filter_Prefix{
				Prefix: &pb.Prefix{
					Key:    ek,
					Prefix: ev,
				},
			}
		} else if f.GetAndOp() != nil {
			nf.Condition = &pb.Filter_AndOp{
				AndOp: &pb.FilterAnd{
					Conditions: c.encryptFilters(f.GetAndOp().Conditions, context),
				},
			}
		} else if f.GetOrOp() != nil {
			nf.Condition = &pb.Filter_OrOp{
				OrOp: &pb.FilterOr{
					Conditions: c.encryptFilters(f.GetOrOp().Conditions, context),
				},
			}
		} else if f.GetNotOp() != nil {
			nf.Condition = &pb.Filter_NotOp{
				NotOp: &pb.FilterNot{
					Condition: c.encryptFilters([]*pb.Filter{f.GetNotOp().Condition}, context)[0],
				},
			}
		} else {
			nf.Condition = f.Condition
		}
		res[i] = nf
	}
	return res
}
