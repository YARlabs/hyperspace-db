#pragma once

#include <cstdint>
#include <cmath>
#include <unordered_map>
#include <vector>
#include <memory>
#include <grpcpp/grpcpp.h>
#include "hyperspace.grpc.pb.h"
#include <google/protobuf/arena.h>

namespace hyperspace {

struct SearchResultRec {
    uint32_t id;
    std::unordered_map<std::string, std::string> metadata;
    std::unordered_map<std::string, ::hyperspace::MetadataValue> typed_metadata;
    double score;
    double distance; // Alias for score
    std::vector<uint8_t> payload; // Sidecar Payload Storage (v3.2)
};

struct CollectionSummaryRec {
    std::string name;
    uint64_t count;
    ::hyperspace::CollectionSchema schema;
};

struct CollectionStats {
    uint64_t count;
    uint64_t indexing_queue;
    uint64_t disk_usage_bytes;
    uint64_t ram_usage_bytes;
    uint64_t active_tasks;
    ::hyperspace::CollectionSchema schema;
};

struct Bm25Params {
    std::string method = "bm25plus";
    float k1 = 1.2f;
    float b = 0.75f;
    float delta = 1.0f;
    std::string language = "english";
};

class HyperspaceClient {
public:
    HyperspaceClient(const std::string& endpoint, const std::string& app_id = "");
    ~HyperspaceClient() = default;

    // Arena Allocation is used internally in Search and BatchSearch to improve deserialization speed
    bool CreateCollection(const std::string& name, const ::hyperspace::CollectionSchema& schema);
    std::vector<CollectionSummaryRec> ListCollections();
    std::string FreezeCollection(const std::string& name);
    std::string UnfreezeCollection(const std::string& name);

    bool Insert(uint32_t id, const std::vector<double>& vector, const std::string& collection = "", const std::vector<uint8_t>& payload = {});
    bool InsertText(uint32_t id, const std::string& text, const std::string& collection = "");
    bool Delete(uint32_t id, const std::string& collection = "");
    bool BatchInsert(const std::vector<uint32_t>& ids, const std::vector<std::vector<double>>& vectors, const std::string& collection = "");
    std::vector<double> Vectorize(const std::string& text, const std::string& metric = "l2");
    std::vector<SearchResultRec> Search(const std::vector<double>& vector, int top_k = 10, const std::string& collection = "", const std::string& hybrid_query = "", float hybrid_alpha = 0.0, const Bm25Params* bm25 = nullptr, uint32_t mrl_dimension = 0, bool use_wasserstein = false, bool include_payload = false, const std::unordered_map<std::string, float>& component_weights = {});
    std::vector<std::vector<SearchResultRec>> SearchBatch(const std::vector<std::vector<double>>& vectors, int top_k = 10, const std::string& collection = "");
    std::vector<SearchResultRec> SearchText(const std::string& text, int top_k = 10, const std::string& collection = "", float hybrid_alpha = 0.0, const Bm25Params* bm25 = nullptr);
    std::unordered_map<std::string, std::vector<SearchResultRec>> SearchMultiCollection(const std::vector<std::string>& collections, const std::vector<double>& query, int top_k = 10);
    
    // Admin & Ops API
    bool GetCollectionStats(const std::string& name, CollectionStats& stats_out);
    bool Exists(const std::string& name);
    bool UpdateCollection(const std::string& name, uint32_t ef_search = 0, uint32_t ef_construction = 0, uint32_t m = 0);
    bool CreateSnapshot();
    bool Vacuum();
    bool RebuildIndex(const std::string& name);
    bool TriggerReconsolidation(const std::string& collection, const std::vector<double>& target, double lr);
    
    // Graph API
    bool GetNode(const std::string& collection, uint32_t id, uint32_t layer, ::hyperspace::GraphNode& node_out);
    bool GetNeighbors(const std::string& collection, uint32_t id, uint32_t layer, uint32_t limit, uint32_t offset, ::hyperspace::GetNeighborsResponse& resp_out);
    bool GetSubsumptionTree(const std::string& collection, uint32_t root_id, uint32_t max_depth, ::hyperspace::GetSubsumptionTreeResponse& resp_out);
    
    // Sync & Events
    bool GetDigest(const std::string& collection, ::hyperspace::DigestResponse& resp_out);
    std::unique_ptr<::grpc::ClientReader<::hyperspace::SystemStats>> Monitor();
    std::unique_ptr<::grpc::ClientReader<::hyperspace::EventMessage>> SubscribeToEvents(const std::vector<::hyperspace::EventType>& types, const std::string& collection = "");
    
    // Sync API signatures
    bool SyncHandshake(const std::string& collection, const std::vector<uint64_t>& client_buckets, uint64_t client_logical_clock, uint64_t client_count, std::vector<uint32_t>& out_diff_buckets);
    std::unique_ptr<::grpc::ClientReader<::hyperspace::SyncVectorData>> SyncPull(const std::string& collection, const std::vector<uint32_t>& bucket_indices);
    std::unique_ptr<::grpc::ClientWriter<::hyperspace::SyncVectorData>> SyncPush(::hyperspace::SyncPushResponse* resp_out);

    // Advanced Data Ops
    std::vector<::hyperspace::VectorData> GetPoints(const std::vector<uint32_t>& ids, const std::string& collection = "");
    bool UpdatePayload(uint32_t id, const std::unordered_map<std::string, std::string>& metadata, const std::unordered_map<std::string, ::hyperspace::MetadataValue>& typed_metadata = {}, const std::string& collection = "");
    std::vector<::hyperspace::VectorData> Scroll(uint32_t limit, uint32_t offset = 0, const std::vector<::hyperspace::Filter>& filters = {}, const std::string& collection = "");
    uint64_t Count(const std::vector<::hyperspace::Filter>& filters = {}, const std::string& collection = "");
    std::string HealthCheck();

private:
    std::unique_ptr<::hyperspace::Database::Stub> stub_;
    std::string app_id_;
};

} // namespace hyperspace
