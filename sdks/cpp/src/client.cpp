#include <hyperspace/client.hpp>
#include <grpcpp/grpcpp.h>
#include "hyperspace.grpc.pb.h"

namespace hyperspace {

using grpc::Channel;
using grpc::ClientContext;
using grpc::Status;
using hyperspace_grpc::Database;

HyperspaceClient::HyperspaceClient(const std::string& endpoint, const std::string& app_id)
    : app_id_(app_id) {
    auto channel = grpc::CreateChannel(endpoint, grpc::InsecureChannelCredentials());
    stub_ = Database::NewStub(channel);
}

bool HyperspaceClient::CreateCollection(const std::string& name, int dimension, const std::string& metric) {
    hyperspace_grpc::CreateCollectionRequest request;
    request.set_name(name);
    request.set_dimension(dimension);
    request.set_metric(metric);

    hyperspace_grpc::Empty response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->CreateCollection(&context, request, &response);
    return status.ok();
}

bool HyperspaceClient::Insert(uint32_t id, const std::vector<double>& vector, const std::string& collection) {
    hyperspace_grpc::InsertRequest request;
    request.set_id(id);
    request.set_collection(collection);
    
    // Copy vector data
    auto* vec_data = request.mutable_vector();
    vec_data->mutable_values()->Add(vector.begin(), vector.end());

    hyperspace_grpc::Empty response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->Insert(&context, request, &response);
    return status.ok();
}

std::vector<SearchResult> HyperspaceClient::Search(const std::vector<double>& vector, int top_k, const std::string& collection) {
    hyperspace_grpc::SearchRequest request;
    request.set_top_k(top_k);
    request.set_collection(collection);

    auto* vec_data = request.mutable_vector();
    vec_data->mutable_values()->Add(vector.begin(), vector.end());

    // Arena allocation for fast protobuf deserialization (Task 2.2 requirement)
    google::protobuf::Arena arena;
    auto* response = google::protobuf::Arena::CreateMessage<hyperspace_grpc::SearchResponse>(&arena);

    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->Search(&context, request, response);
    std::vector<SearchResult> output;

    if (status.ok()) {
        output.reserve(response->results_size());
        for (const auto& res : response->results()) {
            SearchResult s;
            s.id = res.id();
            s.score = res.score();
            
            if (res.has_vector()) {
                const auto& vec = res.vector().values();
                s.vector.assign(vec.begin(), vec.end());
            }

            // Only copy metadata if present
            for (const auto& [k, v] : res.metadata()) {
                s.metadata[k] = v; // Requires parsing if v is TypedMetadataValue
                // (Note: in a full implementation, we'd handle TypedMetadata properly based on the proto)
            }
            output.push_back(s);
        }
    }

    return output;
}

} // namespace hyperspace
