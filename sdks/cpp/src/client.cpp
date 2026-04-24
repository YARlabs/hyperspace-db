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

std::vector<SearchResult> HyperspaceClient::Search(const std::vector<double>& vector, int top_k, const std::string& collection, const std::string& hybrid_query, float hybrid_alpha, const Bm25Options* bm25) {
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
