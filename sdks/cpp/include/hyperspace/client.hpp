#pragma once

#include <string>
#include <vector>
#include <memory>
#include <unordered_map>
#include <grpcpp/grpcpp.h>
#include "hyperspace.grpc.pb.h"
#include <google/protobuf/arena.h>

namespace hyperspace {

struct SearchResult {
    uint32_t id;
    std::unordered_map<std::string, std::string> metadata;
    std::vector<double> vector;
    double score;
};

struct CollectionSummary {
    std::string name;
    uint64_t count;
    uint32_t dimension;
    std::string metric;
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
    std::vector<SearchResult> Search(const std::vector<double>& vector, int top_k = 10, const std::string& collection = "", const std::string& hybrid_query = "", float hybrid_alpha = 0.0);
    std::vector<std::vector<SearchResult>> SearchBatch(const std::vector<std::vector<double>>& vectors, int top_k = 10, const std::string& collection = "");
    std::vector<SearchResult> SearchText(const std::string& text, int top_k = 10, const std::string& collection = "", float hybrid_alpha = 0.0);
    
    // Sync API signatures
    bool SyncHandshake(const std::string& collection, const std::vector<uint64_t>& client_buckets, uint64_t client_logical_clock, uint64_t client_count, std::vector<uint32_t>& out_diff_buckets);
    bool SyncPull(const std::string& collection, const std::vector<uint32_t>& bucket_indices);
    bool SyncPush(const std::string& collection);

private:
    std::unique_ptr<hyperspace_grpc::Database::Stub> stub_;
    std::string app_id_;
};

} // namespace hyperspace
