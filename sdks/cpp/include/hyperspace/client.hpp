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

struct SearchResult {
    uint32_t id;
    std::unordered_map<std::string, std::string> metadata;
    std::vector<double> vector;
    double score;
    double distance; // Alias for score
};

struct CollectionSummary {
    std::string name;
    uint64_t count;
    uint32_t dimension;
    std::string metric;
};

struct CollectionStats {
    uint64_t count;
    uint32_t dimension;
    std::string metric;
    uint64_t indexing_queue;
};

struct Bm25Options {
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
    bool CreateCollection(const std::string& name, int dimension, const std::string& metric = "cosine");
    std::vector<CollectionSummary> ListCollections();

    bool Insert(uint32_t id, const std::vector<double>& vector, const std::string& collection = "");
    bool InsertText(uint32_t id, const std::string& text, const std::string& collection = "");
    bool Delete(uint32_t id, const std::string& collection = "");
    bool BatchInsert(const std::vector<uint32_t>& ids, const std::vector<std::vector<double>>& vectors, const std::string& collection = "");
    std::vector<double> Vectorize(const std::string& text, const std::string& metric = "l2");
    std::vector<SearchResult> Search(const std::vector<double>& vector, int top_k = 10, const std::string& collection = "", const std::string& hybrid_query = "", float hybrid_alpha = 0.0, const Bm25Options* bm25 = nullptr, uint32_t mrl_dimension = 0, bool use_wasserstein = false);
    std::vector<std::vector<SearchResult>> SearchBatch(const std::vector<std::vector<double>>& vectors, int top_k = 10, const std::string& collection = "");
    std::vector<SearchResult> SearchText(const std::string& text, int top_k = 10, const std::string& collection = "", float hybrid_alpha = 0.0, const Bm25Options* bm25 = nullptr);
    std::unordered_map<std::string, std::vector<SearchResult>> SearchMultiCollection(const std::vector<std::string>& collections, const std::vector<double>& query, int top_k = 10);
    
    // Admin & Ops API
    bool GetCollectionStats(const std::string& name, CollectionStats& stats_out);
    bool Exists(const std::string& name);
    bool UpdateCollection(const std::string& name);
    bool CreateSnapshot();
    bool Vacuum();
    bool RebuildIndex(const std::string& name);
    bool TriggerReconsolidation(const std::string& collection, const std::vector<double>& target, double lr);
    
    // Graph API
    bool GetNode(const std::string& collection, uint32_t id, uint32_t layer, ::hyperspace::GraphNode& node_out);
    bool GetNeighbors(const std::string& collection, uint32_t id, uint32_t layer, uint32_t limit, uint32_t offset, ::hyperspace::GetNeighborsResponse& resp_out);
    
    // Sync & Events
    bool GetDigest(const std::string& collection, ::hyperspace::DigestResponse& resp_out);
    std::unique_ptr<::grpc::ClientReader<::hyperspace::SystemStats>> Monitor();
    std::unique_ptr<::grpc::ClientReader<::hyperspace::EventMessage>> SubscribeToEvents(const std::vector<::hyperspace::EventType>& types, const std::string& collection = "");
    
    // Sync API signatures
    bool SyncHandshake(const std::string& collection, const std::vector<uint64_t>& client_buckets, uint64_t client_logical_clock, uint64_t client_count, std::vector<uint32_t>& out_diff_buckets);
    bool SyncPull(const std::string& collection, const std::vector<uint32_t>& bucket_indices);
    bool SyncPush(const std::string& collection);

private:
    std::unique_ptr<::hyperspace::Database::Stub> stub_;
    std::string app_id_;
};

} // namespace hyperspace
