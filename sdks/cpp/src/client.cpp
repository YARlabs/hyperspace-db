#include <hyperspace/client.hpp>
#include <grpcpp/grpcpp.h>
#include "hyperspace.grpc.pb.h"

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

bool HyperspaceClient::CreateCollection(const std::string& name, const ::hyperspace::CollectionSchema& schema) {
    ::hyperspace::CreateCollectionRequest request;
    request.set_name(name);
    *request.mutable_schema() = schema;

    ::hyperspace::StatusResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->CreateCollection(&context, request, &response);
    return status.ok();
}

std::vector<CollectionSummaryRec> HyperspaceClient::ListCollections() {
    ::hyperspace::Empty request;
    ::hyperspace::ListCollectionsResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->ListCollections(&context, request, &response);
    std::vector<CollectionSummaryRec> output;
    if (status.ok()) {
        output.reserve(response.collections_size());
        for (const auto& c : response.collections()) {
            output.push_back({c.name(), (uint64_t)c.count(), c.schema()});
        }
    }
    return output;
}

std::string HyperspaceClient::FreezeCollection(const std::string& name) {
    ::hyperspace::FreezeCollectionRequest request;
    request.set_name(name);

    ::hyperspace::StatusResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->FreezeCollection(&context, request, &response);
    if (status.ok()) {
        return response.status();
    }
    return "Error: " + status.error_message();
}

std::string HyperspaceClient::UnfreezeCollection(const std::string& name) {
    ::hyperspace::UnfreezeCollectionRequest request;
    request.set_name(name);

    ::hyperspace::StatusResponse response;
    ClientContext context;
    if (!app_id_.empty()) {
        context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->UnfreezeCollection(&context, request, &response);
    if (status.ok()) {
        return response.status();
    }
    return "Error: " + status.error_message();
}


bool HyperspaceClient::Insert(uint32_t id, const std::vector<double>& vector, const std::string& collection, const std::vector<uint8_t>& payload) {
    ::hyperspace::InsertRequest request;
    request.set_id(id);
    request.set_collection(collection);
    for (double v : vector) request.add_vector(v);
    if (!payload.empty()) {
        request.set_payload(payload.data(), payload.size());
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
    ::hyperspace::BatchInsertRequest request;
    request.set_collection(collection);
    for (size_t i = 0; i < ids.size(); ++i) {
        auto* v = request.add_vectors();
        v->set_id(ids[i]);
        for (double val : vectors[i]) v->add_vector(val);
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
        for (double v : response.vector()) output.push_back(v);
    }
    return output;
}

std::vector<SearchResultRec> HyperspaceClient::Search(const std::vector<double>& vector, int top_k, const std::string& collection, const std::string& hybrid_query, float hybrid_alpha, const Bm25Params* bm25, uint32_t mrl_dimension, bool use_wasserstein, bool include_payload, const std::unordered_map<std::string, float>& component_weights, bool use_wave, const std::optional<float>& restart_factor) {
    ::hyperspace::SearchRequest request;
    request.set_collection(collection);
    request.set_top_k(top_k);
    for (double v : vector) request.add_vector(v);
    if (!hybrid_query.empty()) request.set_hybrid_query(hybrid_query);
    if (hybrid_alpha != 0.0f) request.set_hybrid_alpha(hybrid_alpha);
    if (bm25) {
        auto* p = request.mutable_bm25_options();
        p->set_method(bm25->method);
        p->set_k1(bm25->k1);
        p->set_b(bm25->b);
        p->set_delta(bm25->delta);
        p->set_language(bm25->language);
    }
    if (mrl_dimension != 0) request.set_mrl_dimension(mrl_dimension);
    request.set_use_wasserstein(use_wasserstein);
    request.set_include_payload(include_payload);
    for (auto const& [k, v] : component_weights) {
        (*request.mutable_component_weights())[k] = v;
    }
    request.set_use_wave(use_wave);
    if (restart_factor.has_value()) {
        (*request.mutable_filter())["wave_restart_factor"] = std::to_string(restart_factor.value());
    }

    ::hyperspace::SearchResponse response;
    ClientContext context;
    if (!app_id_.empty()) context.AddMetadata("x-api-key", app_id_);

    Status status = stub_->Search(&context, request, &response);
    std::vector<SearchResultRec> output;
    if (status.ok()) {
        for (const auto& res : response.results()) {
            SearchResultRec s;
            s.id = res.id();
            s.score = res.distance();
            s.distance = res.distance();
            for (auto const& it : res.metadata()) s.metadata[it.first] = it.second;
            for (auto const& it : res.typed_metadata()) s.typed_metadata[it.first] = it.second;
            if (!res.payload().empty()) {
                s.payload.assign(res.payload().begin(), res.payload().end());
            }
            output.push_back(s);
        }
    }
    return output;
}

std::vector<std::vector<SearchResultRec>> HyperspaceClient::SearchBatch(const std::vector<std::vector<double>>& vectors, int top_k, const std::string& collection) {
    ::hyperspace::BatchSearchRequest request;
    for (const auto& v : vectors) {
        auto* s = request.add_searches();
        s->set_top_k(top_k);
        s->set_collection(collection);
        for (double val : v) s->add_vector(val);
    }

    ::hyperspace::BatchSearchResponse response;
    ClientContext context;
    if (!app_id_.empty()) context.AddMetadata("x-api-key", app_id_);

    Status status = stub_->SearchBatch(&context, request, &response);
    std::vector<std::vector<SearchResultRec>> output;
    if (status.ok()) {
        for (const auto& search_resp : response.responses()) {
            std::vector<SearchResultRec> sub;
            for (const auto& res : search_resp.results()) {
                SearchResultRec s;
                s.id = res.id();
                s.score = res.distance();
                s.distance = res.distance();
                for (auto const& it : res.metadata()) s.metadata[it.first] = it.second;
                for (auto const& it : res.typed_metadata()) s.typed_metadata[it.first] = it.second;
                sub.push_back(s);
            }
            output.push_back(sub);
        }
    }
    return output;
}

std::vector<SearchResultRec> HyperspaceClient::SearchText(const std::string& text, int top_k, const std::string& collection, float hybrid_alpha, const Bm25Params* bm25) {
    ::hyperspace::SearchTextRequest request;
    request.set_text(text);
    request.set_top_k(top_k);
    request.set_collection(collection);
    if (hybrid_alpha != 0.0f) request.set_hybrid_alpha(hybrid_alpha);
    if (bm25) {
        auto* p = request.mutable_bm25_options();
        p->set_method(bm25->method);
        p->set_k1(bm25->k1);
        p->set_b(bm25->b);
        p->set_delta(bm25->delta);
        p->set_language(bm25->language);
    }

    ::hyperspace::SearchResponse response;
    ClientContext context;
    if (!app_id_.empty()) context.AddMetadata("x-api-key", app_id_);

    Status status = stub_->SearchText(&context, request, &response);
    std::vector<SearchResultRec> output;
    if (status.ok()) {
        for (const auto& res : response.results()) {
            SearchResultRec s;
            s.id = res.id();
            s.score = res.distance();
            s.distance = res.distance();
            for (auto const& it : res.metadata()) s.metadata[it.first] = it.second;
            for (auto const& it : res.typed_metadata()) s.typed_metadata[it.first] = it.second;
            output.push_back(s);
        }
    }
    return output;
}

std::unordered_map<std::string, std::vector<SearchResultRec>> HyperspaceClient::SearchMultiCollection(const std::vector<std::string>& collections, const std::vector<double>& query, int top_k) {
    ::hyperspace::SearchMultiCollectionRequest request;
    for (const auto& c : collections) request.add_collections(c);
    for (double v : query) request.add_vector(v);
    request.set_top_k(top_k);

    ::hyperspace::SearchMultiCollectionResponse response;
    ClientContext context;
    if (!app_id_.empty()) context.AddMetadata("x-api-key", app_id_);

    Status status = stub_->SearchMultiCollection(&context, request, &response);
    std::unordered_map<std::string, std::vector<SearchResultRec>> output;
    if (status.ok()) {
        for (const auto& it : response.responses()) {
            std::vector<SearchResultRec> sub;
            for (const auto& res : it.second.results()) {
                SearchResultRec s;
                s.id = res.id();
                s.score = res.distance();
                s.distance = res.distance();
                for (auto const& m : res.metadata()) s.metadata[m.first] = m.second;
                for (auto const& m : res.typed_metadata()) s.typed_metadata[m.first] = m.second;
                sub.push_back(s);
            }
            output[it.first] = sub;
        }
    }
    return output;
}

bool HyperspaceClient::UpdateCollection(const std::string& name, uint32_t ef_search, uint32_t ef_construction, uint32_t m) {
    ::hyperspace::ConfigUpdate request;
    request.set_collection(name);
    if (ef_search != 0) request.set_ef_search(ef_search);
    if (ef_construction != 0) request.set_ef_construction(ef_construction);
    if (m != 0) request.set_m(m);

    ::hyperspace::StatusResponse response;
    ClientContext context;
    if (!app_id_.empty()) context.AddMetadata("x-api-key", app_id_);

    Status status = stub_->Configure(&context, request, &response);
    return status.ok();
}

bool HyperspaceClient::GetCollectionStats(const std::string& name, CollectionStats& stats_out) {
    ::hyperspace::CollectionStatsRequest request;
    request.set_name(name);
    ::hyperspace::CollectionStatsResponse response;
    ClientContext context;
    if (!app_id_.empty()) context.AddMetadata("x-api-key", app_id_);
    Status status = stub_->GetCollectionStats(&context, request, &response);
    if (status.ok()) {
        stats_out.count = response.count();
        stats_out.indexing_queue = response.indexing_queue();
        stats_out.disk_usage_bytes = response.disk_usage_bytes();
        stats_out.ram_usage_bytes = response.ram_usage_bytes();
        stats_out.active_tasks = response.active_tasks();
        stats_out.schema = response.schema();
        return true;
    }
    return false;
}

bool HyperspaceClient::Exists(const std::string& name) {
    CollectionStats s;
    return GetCollectionStats(name, s);
}

bool HyperspaceClient::CreateSnapshot() {
    ::hyperspace::Empty req;
    ::hyperspace::StatusResponse resp;
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    return stub_->TriggerSnapshot(&ctx, req, &resp).ok();
}

bool HyperspaceClient::Vacuum() {
    ::hyperspace::Empty req;
    ::hyperspace::StatusResponse resp;
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    return stub_->TriggerVacuum(&ctx, req, &resp).ok();
}

bool HyperspaceClient::RebuildIndex(const std::string& name) {
    ::hyperspace::RebuildIndexRequest req;
    req.set_name(name);
    ::hyperspace::StatusResponse resp;
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    return stub_->RebuildIndex(&ctx, req, &resp).ok();
}

bool HyperspaceClient::TriggerReconsolidation(const std::string& col, const std::vector<double>& target, double lr) {
    ::hyperspace::ReconsolidationRequest req;
    req.set_collection(col);
    for (double v : target) req.add_target_vector(v);
    req.set_learning_rate(lr);
    ::hyperspace::StatusResponse resp;
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    return stub_->TriggerReconsolidation(&ctx, req, &resp).ok();
}

bool HyperspaceClient::GetNode(const std::string& col, uint32_t id, uint32_t layer, ::hyperspace::GraphNode& node_out) {
    ::hyperspace::GetNodeRequest req;
    req.set_collection(col);
    req.set_id(id);
    req.set_layer(layer);
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    return stub_->GetNode(&ctx, req, &node_out).ok();
}

bool HyperspaceClient::GetNeighbors(const std::string& col, uint32_t id, uint32_t layer, uint32_t limit, uint32_t offset, ::hyperspace::GetNeighborsResponse& resp_out) {
    ::hyperspace::GetNeighborsRequest req;
    req.set_collection(col);
    req.set_id(id);
    req.set_layer(layer);
    req.set_limit(limit);
    req.set_offset(offset);
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    return stub_->GetNeighbors(&ctx, req, &resp_out).ok();
}

bool HyperspaceClient::GetDigest(const std::string& col, ::hyperspace::DigestResponse& resp_out) {
    ::hyperspace::DigestRequest req;
    req.set_collection(col);
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    return stub_->GetDigest(&ctx, req, &resp_out).ok();
}

std::unique_ptr<::grpc::ClientReader<::hyperspace::SystemStats>> HyperspaceClient::Monitor() {
    ::hyperspace::MonitorRequest req;
    auto ctx = std::make_unique<ClientContext>();
    if (!app_id_.empty()) ctx->AddMetadata("x-api-key", app_id_);
    return stub_->Monitor(ctx.get(), req);
}

std::unique_ptr<::grpc::ClientReader<::hyperspace::EventMessage>> HyperspaceClient::SubscribeToEvents(const std::vector<::hyperspace::EventType>& types, const std::string& col) {
    ::hyperspace::EventSubscriptionRequest req;
    if (!col.empty()) req.set_collection(col);
    for (auto t : types) req.add_types(t);
    auto ctx = std::make_unique<ClientContext>();
    if (!app_id_.empty()) ctx->AddMetadata("x-api-key", app_id_);
    return stub_->SubscribeToEvents(ctx.get(), req);
}

std::vector<::hyperspace::VectorData> HyperspaceClient::GetPoints(const std::vector<uint32_t>& ids, const std::string& col) {
    ::hyperspace::GetPointsRequest req;
    req.set_collection(col);
    for (uint32_t id : ids) req.add_ids(id);
    ::hyperspace::GetPointsResponse resp;
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    Status s = stub_->GetPoints(&ctx, req, &resp);
    std::vector<::hyperspace::VectorData> out;
    if (s.ok()) {
        for (const auto& p : resp.points()) out.push_back(p);
    }
    return out;
}

bool HyperspaceClient::UpdatePayload(uint32_t id, const std::unordered_map<std::string, std::string>& meta, const std::unordered_map<std::string, ::hyperspace::MetadataValue>& typed_meta, const std::string& col) {
    ::hyperspace::UpdatePayloadRequest req;
    req.set_id(id);
    req.set_collection(col);
    auto* m = req.mutable_metadata();
    for (const auto& [k, v] : meta) (*m)[k] = v;
    auto* tm = req.mutable_typed_metadata();
    for (const auto& [k, v] : typed_meta) (*tm)[k] = v;
    ::hyperspace::StatusResponse resp;
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    return stub_->UpdatePayload(&ctx, req, &resp).ok();
}

std::vector<::hyperspace::VectorData> HyperspaceClient::Scroll(uint32_t limit, uint32_t offset, const std::vector<::hyperspace::Filter>& filters, const std::string& col) {
    ::hyperspace::ScrollRequest req;
    req.set_collection(col);
    req.set_limit(limit);
    req.set_offset(offset);
    for (const auto& f : filters) *req.add_filters() = f;
    ::hyperspace::ScrollResponse resp;
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    Status s = stub_->Scroll(&ctx, req, &resp);
    std::vector<::hyperspace::VectorData> out;
    if (s.ok()) {
        for (const auto& p : resp.points()) out.push_back(p);
    }
    return out;
}

uint64_t HyperspaceClient::Count(const std::vector<::hyperspace::Filter>& filters, const std::string& col) {
    ::hyperspace::CountRequest req;
    req.set_collection(col);
    for (const auto& f : filters) *req.add_filters() = f;
    ::hyperspace::CountResponse resp;
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    Status s = stub_->Count(&ctx, req, &resp);
    if (s.ok()) return resp.count();
    return 0;
}

bool HyperspaceClient::SyncHandshake(const std::string& collection, const std::vector<uint64_t>& client_buckets, uint64_t client_logical_clock, uint64_t client_count, std::vector<uint32_t>& out_diff_buckets) {
    ::hyperspace::SyncHandshakeRequest req;
    req.set_collection(collection);
    for (uint64_t b : client_buckets) req.add_client_buckets(b);
    req.set_client_logical_clock(client_logical_clock);
    req.set_client_count(client_count);
    ::hyperspace::SyncHandshakeResponse resp;
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    Status s = stub_->SyncHandshake(&ctx, req, &resp);
    if (s.ok()) {
        for (const auto& b : resp.diff_buckets()) out_diff_buckets.push_back(b.bucket_index());
        return true;
    }
    return false;
}

std::unique_ptr<::grpc::ClientReader<::hyperspace::SyncVectorData>> HyperspaceClient::SyncPull(const std::string& collection, const std::vector<uint32_t>& bucket_indices) {
    ::hyperspace::SyncPullRequest req;
    req.set_collection(collection);
    for (uint32_t idx : bucket_indices) req.add_bucket_indices(idx);
    auto ctx = std::make_unique<ClientContext>();
    if (!app_id_.empty()) ctx->AddMetadata("x-api-key", app_id_);
    return stub_->SyncPull(ctx.get(), req);
}

std::unique_ptr<::grpc::ClientWriter<::hyperspace::SyncVectorData>> HyperspaceClient::SyncPush(::hyperspace::SyncPushResponse* resp_out) {
    auto ctx = std::make_unique<ClientContext>();
    if (!app_id_.empty()) ctx->AddMetadata("x-api-key", app_id_);
    return stub_->SyncPush(ctx.get(), resp_out); 
}

std::string HyperspaceClient::HealthCheck() {
    ::hyperspace::Empty req;
    ::hyperspace::HealthCheckResponse resp;
    ClientContext ctx;
    if (!app_id_.empty()) ctx.AddMetadata("x-api-key", app_id_);
    Status s = stub_->HealthCheck(&ctx, req, &resp);
    if (s.ok()) return resp.status();
    return "unreachable";
}

} // namespace hyperspace
