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

class HyperspaceClient {
public:
    HyperspaceClient(const std::string& endpoint, const std::string& app_id = "");
    ~HyperspaceClient() = default;

    // Arena Allocation is used internally in Search and BatchSearch to improve deserialization speed
    bool CreateCollection(const std::string& name, int dimension, const std::string& metric = "cosine");
    bool Insert(uint32_t id, const std::vector<double>& vector, const std::string& collection = "");
    std::vector<SearchResult> Search(const std::vector<double>& vector, int top_k = 10, const std::string& collection = "");
    
    // Sync API signatures
    bool SyncHandshake(const std::string& collection, const std::vector<uint64_t>& client_buckets, uint64_t client_logical_clock, uint64_t client_count, std::vector<uint32_t>& out_diff_buckets);
    bool SyncPull(const std::string& collection, const std::vector<uint32_t>& bucket_indices);
    bool SyncPush(const std::string& collection);

private:
    std::unique_ptr<hyperspace_grpc::Database::Stub> stub_;
    std::string app_id_;
};

} // namespace hyperspace
