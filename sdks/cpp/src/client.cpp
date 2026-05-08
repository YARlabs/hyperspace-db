#include <hyperspace/client.hpp>
#include <grpcpp/grpcpp.h>
#include "hyperspace.grpc.pb.h"
#include <google/protobuf/arena.h>

namespace hyperspace {

using grpc::Channel;
using grpc::ClientContext;
using grpc::Status;
using ::hyperspace::Database;

HyperspaceClient::HyperspaceClient(const std::string& endpoint, const std::string& app_id)
    : app_id_(app_id) {
    auto channel = grpc::CreateChannel(endpoint, grpc::InsecureChannelCredentials());
    stub_ = Database::NewStub(channel);
}

bool HyperspaceClient::CreateCollection(const std::string& name, int dimension, const std::string& metric) {
    ::hyperspace::CreateCollectionRequest request;
    request.set_name(name);
    request.set_dimension(dimension);
    request.set_metric(metric);

    ::hyperspace::StatusResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->CreateCollection(&context, request, &response);
    return status.ok();
}

std::vector<CollectionSummary> HyperspaceClient::ListCollections() {
    ::hyperspace::Empty request;
    ::hyperspace::ListCollectionsResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->ListCollections(&context, request, &response);
    std::vector<CollectionSummary> output;
    if (status.ok()) {
        output.reserve(response.collections_size());
        for (const auto& c : response.collections()) {
            output.push_back({c.name(), (uint64_t)c.count(), (uint32_t)c.dimension(), c.metric()});
        }
    }
    return output;
}


bool HyperspaceClient::Insert(uint32_t id, const std::vector<double>& vector, const std::string& collection) {
    ::hyperspace::InsertRequest request;
    request.set_id(id);
    request.set_collection(collection);
    
    for (double v : vector) {
        request.add_vector(v);
    }

    ::hyperspace::InsertResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->Insert(&context, request, &response);
    return status.ok() && response.success();
}

bool HyperspaceClient::InsertText(uint32_t id, const std::string& text, const std::string& collection) {
    ::hyperspace::InsertTextRequest request;
    request.set_id(id);
    request.set_text(text);
    request.set_collection(collection);

    ::hyperspace::InsertResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->InsertText(&context, request, &response);
    return status.ok() && response.success();
}

bool HyperspaceClient::Delete(uint32_t id, const std::string& collection) {
    ::hyperspace::DeleteRequest request;
    request.set_id(id);
    request.set_collection(collection);

    ::hyperspace::DeleteResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->Delete(&context, request, &response);
    return status.ok() && response.success();
}

bool HyperspaceClient::BatchInsert(const std::vector<uint32_t>& ids, const std::vector<std::vector<double>>& vectors, const std::string& collection) {
    if (ids.size() != vectors.size()) return false;
    
    ::hyperspace::BatchInsertRequest request;
    request.set_collection(collection);
    
    for (size_t i = 0; i < ids.size(); ++i) {
        auto* v = request.add_vectors();
        v->set_id(ids[i]);
        for (double val : vectors[i]) {
            v->add_vector(val);
        }
    }

    ::hyperspace::InsertResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }
    Status status = stub_->BatchInsert(&context, request, &response);
    return status.ok() && response.success();
}

std::vector<double> HyperspaceClient::Vectorize(const std::string& text, const std::string& metric) {
    ::hyperspace::VectorizeRequest request;
    request.set_text(text);
    request.set_metric(metric);

    ::hyperspace::VectorizeResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->Vectorize(&context, request, &response);
    std::vector<double> output;
    if (status.ok()) {
        output.assign(response.vector().begin(), response.vector().end());
    }
    return output;
}

std::vector<SearchResult> HyperspaceClient::Search(const std::vector<double>& vector, int top_k, const std::string& collection, const std::string& hybrid_query, float hybrid_alpha, const Bm25Options* bm25, uint32_t mrl_dimension, bool use_wasserstein) {
    ::hyperspace::SearchRequest request;
    request.set_top_k(top_k);
    request.set_collection(collection);

    for (double v : vector) {
        request.add_vector(v);
    }
    
    if (!hybrid_query.empty()) {
        request.set_hybrid_query(hybrid_query);
    }
    if (hybrid_alpha != 0.0f) {
        request.set_hybrid_alpha(hybrid_alpha);
    }
    
    if (bm25) {
        auto* proto_bm25 = request.mutable_bm25_options();
        proto_bm25->set_method(bm25->method);
        proto_bm25->set_k1(bm25->k1);
        proto_bm25->set_b(bm25->b);
        proto_bm25->set_delta(bm25->delta);
        proto_bm25->set_language(bm25->language);
    }
    
    if (mrl_dimension != 0) {
        request.set_mrl_dimension(mrl_dimension);
    }
    if (use_wasserstein) {
        request.set_use_wasserstein(use_wasserstein);
    }

    google::protobuf::Arena arena;
    auto* response = google::protobuf::Arena::CreateMessage<::hyperspace::SearchResponse>(&arena);

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
            s.score = res.distance();
            s.distance = res.distance();
            
            for (double v : res.vector()) {
                s.vector.push_back(v);
            }

            for (auto const& it : res.metadata()) {
                s.metadata[it.first] = it.second;
            }
            output.push_back(s);
        }
    }

    return output;
}

std::vector<std::vector<SearchResult>> HyperspaceClient::SearchBatch(const std::vector<std::vector<double>>& vectors, int top_k, const std::string& collection) {
    ::hyperspace::BatchSearchRequest request;
    for (const auto& v : vectors) {
        auto* s = request.add_searches();
        s->set_top_k(top_k);
        s->set_collection(collection);
        for (double val : v) s->add_vector(val);
    }

    google::protobuf::Arena arena;
    auto* response = google::protobuf::Arena::CreateMessage<::hyperspace::BatchSearchResponse>(&arena);

    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->SearchBatch(&context, request, response);
    std::vector<std::vector<SearchResult>> output;

    if (status.ok()) {
        output.reserve(response->responses_size());
        for (const auto& search_resp : response->responses()) {
            std::vector<SearchResult> sub_results;
            sub_results.reserve(search_resp.results_size());
            for (const auto& res : search_resp.results()) {
                SearchResult s;
                s.id = res.id();
                s.score = res.distance();
                s.distance = res.distance();
                for (double val : res.vector()) s.vector.push_back(val);
                for (auto const& it : res.metadata()) {
                    s.metadata[it.first] = it.second;
                }
                sub_results.push_back(s);
            }
            output.push_back(sub_results);
        }
    }
    return output;
}

std::vector<SearchResult> HyperspaceClient::SearchText(const std::string& text, int top_k, const std::string& collection, float hybrid_alpha, const Bm25Options* bm25) {
    ::hyperspace::SearchTextRequest request;
    request.set_text(text);
    request.set_top_k(top_k);
    request.set_collection(collection);
    
    if (hybrid_alpha != 0.0f) {
        request.set_hybrid_alpha(hybrid_alpha);
    }

    if (bm25) {
        auto* proto_bm25 = request.mutable_bm25_options();
        proto_bm25->set_method(bm25->method);
        proto_bm25->set_k1(bm25->k1);
        proto_bm25->set_b(bm25->b);
        proto_bm25->set_delta(bm25->delta);
        proto_bm25->set_language(bm25->language);
    }

    google::protobuf::Arena arena;
    auto* response = google::protobuf::Arena::CreateMessage<::hyperspace::SearchResponse>(&arena);

    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->SearchText(&context, request, response);
    std::vector<SearchResult> output;

    if (status.ok()) {
        output.reserve(response->results_size());
        for (const auto& res : response->results()) {
            SearchResult s;
            s.id = res.id();
            s.score = res.distance();
            s.distance = res.distance();
            
            for (auto const& it : res.metadata()) {
                s.metadata[it.first] = it.second;
            }
            output.push_back(s);
        }
    }

    return output;
}

} // namespace hyperspace

std::unordered_map<std::string, std::vector<SearchResult>> HyperspaceClient::SearchMultiCollection(const std::vector<std::string>& collections, const std::vector<double>& query, int top_k) {
    ::hyperspace::SearchMultiCollectionRequest request;
    for (const auto& c : collections) {
        request.add_collections(c);
    }
    for (double v : query) {
        request.add_vector(v);
    }
    request.set_top_k(top_k);

    ::hyperspace::SearchMultiCollectionResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->SearchMultiCollection(&context, request, &response);
    std::unordered_map<std::string, std::vector<SearchResult>> output;

    if (status.ok()) {
        for (const auto& it : response.responses()) {
            std::vector<SearchResult> sub_results;
            sub_results.reserve(it.second.results_size());
            for (const auto& res : it.second.results()) {
                SearchResult s;
                s.id = res.id();
                s.score = res.distance();
                s.distance = res.distance();
                for (auto const& meta_it : res.metadata()) {
                    s.metadata[meta_it.first] = meta_it.second;
                }
                sub_results.push_back(s);
            }
            output[it.first] = sub_results;
        }
    }
    return output;
}

bool HyperspaceClient::GetCollectionStats(const std::string& name, CollectionStats& stats_out) {
    ::hyperspace::CollectionStatsRequest request;
    request.set_name(name);
    
    ::hyperspace::CollectionStatsResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->GetCollectionStats(&context, request, &response);
    if (status.ok()) {
        stats_out.count = response.count();
        stats_out.dimension = response.dimension();
        stats_out.metric = response.metric();
        stats_out.indexing_queue = response.indexing_queue();
        return true;
    }
    return false;
}

bool HyperspaceClient::Exists(const std::string& name) {
    CollectionStats dummy;
    return GetCollectionStats(name, dummy);
}

bool HyperspaceClient::UpdateCollection(const std::string& name) {
    ::hyperspace::ConfigUpdate request;
    request.set_collection(name);
    ::hyperspace::StatusResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }
    Status status = stub_->Configure(&context, request, &response);
    return status.ok() && response.status() == "success";
}

bool HyperspaceClient::CreateSnapshot() {
    ::hyperspace::Empty request;
    ::hyperspace::StatusResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }
    Status status = stub_->TriggerSnapshot(&context, request, &response);
    return status.ok() && response.status() == "success";
}

bool HyperspaceClient::Vacuum() {
    ::hyperspace::Empty request;
    ::hyperspace::StatusResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }
    Status status = stub_->TriggerVacuum(&context, request, &response);
    return status.ok() && response.status() == "success";
}

bool HyperspaceClient::RebuildIndex(const std::string& name) {
    ::hyperspace::RebuildIndexRequest request;
    request.set_name(name);
    ::hyperspace::StatusResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }
    Status status = stub_->RebuildIndex(&context, request, &response);
    return status.ok() && response.status() == "success";
}

bool HyperspaceClient::TriggerReconsolidation(const std::string& collection, const std::vector<double>& target, double lr) {
    ::hyperspace::ReconsolidationRequest request;
    request.set_collection(collection);
    for (double v : target) request.add_target_vector(v);
    request.set_learning_rate(lr);
    ::hyperspace::StatusResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }
    Status status = stub_->TriggerReconsolidation(&context, request, &response);
    return status.ok() && response.status() == "success";
}

bool HyperspaceClient::GetNode(const std::string& collection, uint32_t id, uint32_t layer, ::hyperspace::GraphNode& node_out) {
    ::hyperspace::GetNodeRequest request;
    request.set_collection(collection);
    request.set_id(id);
    request.set_layer(layer);
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }
    Status status = stub_->GetNode(&context, request, &node_out);
    return status.ok();
}

bool HyperspaceClient::GetNeighbors(const std::string& collection, uint32_t id, uint32_t layer, uint32_t limit, uint32_t offset, ::hyperspace::GetNeighborsResponse& resp_out) {
    ::hyperspace::GetNeighborsRequest request;
    request.set_collection(collection);
    request.set_id(id);
    request.set_layer(layer);
    request.set_limit(limit);
    request.set_offset(offset);
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }
    Status status = stub_->GetNeighbors(&context, request, &resp_out);
    return status.ok();
}

bool HyperspaceClient::GetDigest(const std::string& collection, ::hyperspace::DigestResponse& resp_out) {
    ::hyperspace::DigestRequest request;
    request.set_collection(collection);
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }
    Status status = stub_->GetDigest(&context, request, &resp_out);
    return status.ok();
}

std::unique_ptr<::grpc::ClientReader<::hyperspace::SystemStats>> HyperspaceClient::Monitor() {
    ::hyperspace::MonitorRequest request;
    auto context = std::make_unique<ClientContext>();
    if (!app_id_.empty()) {
        context->AddMetadata("x-api-key", app_id_);
    }
    // Note: Caller must manage context lifetime if we returned a reader that depends on it.
    // In this simplified implementation, we return the reader but context management is tricky.
    return stub_->Monitor(context.get(), request);
}

std::unique_ptr<::grpc::ClientReader<::hyperspace::EventMessage>> HyperspaceClient::SubscribeToEvents(const std::vector<::hyperspace::EventType>& types, const std::string& collection) {
    ::hyperspace::EventSubscriptionRequest request;
    if (!collection.empty()) request.set_collection(collection);
    for (auto t : types) request.add_types(static_cast<int32_t>(t));
    auto context = std::make_unique<ClientContext>();
    if (!app_id_.empty()) {
        context->AddMetadata("x-api-key", app_id_);
    }
    return stub_->SubscribeToEvents(context.get(), request);
}


