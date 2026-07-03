#include <hyperspace/client.hpp>
#include <hyperspace/math.hpp>
#include <grpcpp/grpcpp.h>
#include "hyperspace.grpc.pb.h"
#include <iomanip>
#include <sstream>
#include <algorithm>
#include <openssl/evp.h>
#include <openssl/hmac.h>
#include <openssl/rand.h>

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
    return InsertSecure(id, vector, {}, collection, payload);
}

bool HyperspaceClient::InsertSecure(uint32_t id, const std::vector<double>& vector, const std::unordered_map<std::string, std::string>& metadata, const std::string& collection, const std::vector<uint8_t>& payload) {
    std::string metric = "l2";
    {
        auto it = collection_metrics_.find(collection);
        if (it != collection_metrics_.end()) {
            metric = it->second;
        }
    }

    auto context = get_encryption_context(collection, vector.size(), metric);

    std::vector<double> final_vector = vector;
    std::unordered_map<std::string, std::string> final_metadata = metadata;
    std::vector<uint8_t> final_payload = payload;

    if (context) {
        // 1. Noise injection
        double sigma = 0.02;
        {
            auto it = collection_noise_sigmas_.find(collection);
            if (it != collection_noise_sigmas_.end()) {
                sigma = it->second;
            }
        }
        if (sigma > 0.0) {
            final_vector = math::inject_anisotropic_noise(final_vector, context->hmac_key, sigma);
        }

        // 2. Vector projection
        final_vector = project_collection_vector(collection, final_vector, *context, metric);

        // 3. Encrypt payload
        if (!payload.empty()) {
            auto enc = encrypt_payload(payload, context->aes_key);
            if (!enc.empty()) final_payload = enc;
        }

        // 4. Hash metadata
        std::unordered_map<std::string, std::string> hashed_meta;
        for (auto const& [k, v] : final_metadata) {
            hashed_meta[hash_metadata_key(k, context->hmac_key)] = hash_metadata_value(v, context->hmac_key);
        }
        final_metadata = hashed_meta;
    }

    ::hyperspace::InsertRequest request;
    request.set_id(id);
    request.set_collection(collection);
    for (double v : final_vector) request.add_vector(v);
    if (!final_payload.empty()) {
        request.set_payload(final_payload.data(), final_payload.size());
    }
    for (auto const& [k, v] : final_metadata) {
        (*request.mutable_metadata())[k] = v;
    }

    ::hyperspace::InsertResponse response;
    ClientContext grpc_context;
    if (!app_id_.empty()) {
        grpc_context.AddMetadata("x-api-key", app_id_);
    }

    Status status = stub_->Insert(&grpc_context, request, &response);
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
    std::string metric = "l2";
    {
        auto it = collection_metrics_.find(collection);
        if (it != collection_metrics_.end()) {
            metric = it->second;
        }
    }

    auto context = get_encryption_context(collection, vector.size(), metric);

    std::vector<double> final_vector = vector;
    bool final_include_payload = include_payload;

    if (context) {
        // 1. Noise injection
        double sigma = 0.02;
        {
            auto it = collection_noise_sigmas_.find(collection);
            if (it != collection_noise_sigmas_.end()) {
                sigma = it->second;
            }
        }
        if (sigma > 0.0) {
            final_vector = math::inject_anisotropic_noise(final_vector, context->hmac_key, sigma);
        }

        // 2. Vector projection
        final_vector = project_collection_vector(collection, final_vector, *context, metric);

        // 3. Force include payload for decryption
        final_include_payload = true;
    }

    ::hyperspace::SearchRequest request;
    request.set_collection(collection);
    request.set_top_k(top_k);
    for (double v : final_vector) request.add_vector(v);
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
    request.set_include_payload(final_include_payload);
    for (auto const& [k, v] : component_weights) {
        (*request.mutable_component_weights())[k] = v;
    }
    request.set_use_wave(use_wave);
    if (restart_factor.has_value()) {
        (*request.mutable_filter())["wave_restart_factor"] = std::to_string(restart_factor.value());
    }

    ::hyperspace::SearchResponse response;
    ClientContext grpc_context;
    if (!app_id_.empty()) grpc_context.AddMetadata("x-api-key", app_id_);

    Status status = stub_->Search(&grpc_context, request, &response);
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
                std::vector<uint8_t> payload_bytes(res.payload().begin(), res.payload().end());
                if (context) {
                    auto dec = decrypt_payload(payload_bytes, context->aes_key);
                    if (!dec.empty()) {
                        s.payload = dec;
                    } else {
                        s.payload = payload_bytes;
                    }
                } else {
                    s.payload = payload_bytes;
                }
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

std::vector<uint8_t> HyperspaceClient::derive_pbkdf2(const std::vector<uint8_t>& password, const std::vector<uint8_t>& salt, int iterations, int key_len) {
    std::vector<uint8_t> out_key(key_len);
    PKCS5_PBKDF2_HMAC(
        reinterpret_cast<const char*>(password.data()), password.size(),
        salt.data(), salt.size(),
        iterations, EVP_sha256(),
        key_len, out_key.data()
    );
    return out_key;
}

std::pair<std::vector<uint8_t>, std::vector<uint8_t>> HyperspaceClient::derive_keys(const std::string& password, const std::string& collection_name) {
    std::vector<uint8_t> pwd(password.begin(), password.end());
    std::vector<uint8_t> coll(collection_name.begin(), collection_name.end());
    std::vector<uint8_t> salt = math::sha256_hash(coll);

    auto aes_key = derive_pbkdf2(pwd, salt, 100000, 32);
    auto hmac_key = derive_pbkdf2(pwd, salt, 100000, 32);
    return {aes_key, hmac_key};
}

std::vector<uint8_t> HyperspaceClient::encrypt_payload(const std::vector<uint8_t>& plaintext, const std::vector<uint8_t>& aes_key) {
    std::vector<uint8_t> pbkdf2_salt(16);
    RAND_bytes(pbkdf2_salt.data(), 16);

    auto derived_key = derive_pbkdf2(aes_key, pbkdf2_salt, 100000, 32);

    std::vector<uint8_t> iv(12);
    RAND_bytes(iv.data(), 12);

    EVP_CIPHER_CTX* ctx = EVP_CIPHER_CTX_new();
    if (!ctx) return {};

    if (EVP_EncryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL) != 1) {
        EVP_CIPHER_CTX_free(ctx);
        return {};
    }

    if (EVP_EncryptInit_ex(ctx, NULL, NULL, derived_key.data(), iv.data()) != 1) {
        EVP_CIPHER_CTX_free(ctx);
        return {};
    }

    std::vector<uint8_t> ciphertext(plaintext.size());
    int len = 0;
    if (EVP_EncryptUpdate(ctx, ciphertext.data(), &len, plaintext.data(), plaintext.size()) != 1) {
        EVP_CIPHER_CTX_free(ctx);
        return {};
    }

    int ciphertext_len = len;
    if (EVP_EncryptFinal_ex(ctx, ciphertext.data() + len, &len) != 1) {
        EVP_CIPHER_CTX_free(ctx);
        return {};
    }
    ciphertext_len += len;

    std::vector<uint8_t> tag(16);
    if (EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_AEAD_GET_TAG, 16, tag.data()) != 1) {
        EVP_CIPHER_CTX_free(ctx);
        return {};
    }

    EVP_CIPHER_CTX_free(ctx);

    std::vector<uint8_t> result;
    result.reserve(16 + 12 + ciphertext_len + 16);
    result.insert(result.end(), pbkdf2_salt.begin(), pbkdf2_salt.end());
    result.insert(result.end(), iv.begin(), iv.end());
    result.insert(result.end(), ciphertext.begin(), ciphertext.end());
    result.insert(result.end(), tag.begin(), tag.end());
    return result;
}

std::vector<uint8_t> HyperspaceClient::decrypt_payload(const std::vector<uint8_t>& ciphertext, const std::vector<uint8_t>& aes_key) {
    if (ciphertext.size() < 16 + 12 + 16) return {};

    std::vector<uint8_t> pbkdf2_salt(ciphertext.begin(), ciphertext.begin() + 16);
    std::vector<uint8_t> iv(ciphertext.begin() + 16, ciphertext.begin() + 28);
    std::vector<uint8_t> actual_ciphertext(ciphertext.begin() + 28, ciphertext.end() - 16);
    std::vector<uint8_t> tag(ciphertext.end() - 16, ciphertext.end());

    auto derived_key = derive_pbkdf2(aes_key, pbkdf2_salt, 100000, 32);

    EVP_CIPHER_CTX* ctx = EVP_CIPHER_CTX_new();
    if (!ctx) return {};

    if (EVP_DecryptInit_ex(ctx, EVP_aes_256_gcm(), NULL, NULL, NULL) != 1) {
        EVP_CIPHER_CTX_free(ctx);
        return {};
    }

    if (EVP_DecryptInit_ex(ctx, NULL, NULL, derived_key.data(), iv.data()) != 1) {
        EVP_CIPHER_CTX_free(ctx);
        return {};
    }

    std::vector<uint8_t> plaintext(actual_ciphertext.size());
    int len = 0;
    if (EVP_DecryptUpdate(ctx, plaintext.data(), &len, actual_ciphertext.data(), actual_ciphertext.size()) != 1) {
        EVP_CIPHER_CTX_free(ctx);
        return {};
    }

    int plaintext_len = len;
    if (EVP_CIPHER_CTX_ctrl(ctx, EVP_CTRL_AEAD_SET_TAG, 16, tag.data()) != 1) {
        EVP_CIPHER_CTX_free(ctx);
        return {};
    }

    int ret = EVP_DecryptFinal_ex(ctx, plaintext.data() + len, &len);
    EVP_CIPHER_CTX_free(ctx);

    if (ret > 0) {
        plaintext_len += len;
        plaintext.resize(plaintext_len);
        return plaintext;
    }
    return {};
}

std::string HyperspaceClient::hash_metadata_key(const std::string& key, const std::vector<uint8_t>& hmac_key) {
    uint8_t hash[EVP_MAX_MD_SIZE];
    unsigned int hash_len = 0;
    HMAC(EVP_sha256(), hmac_key.data(), hmac_key.size(), reinterpret_cast<const unsigned char*>(key.data()), key.size(), hash, &hash_len);

    std::stringstream ss;
    for (unsigned int i = 0; i < hash_len; ++i) {
        ss << std::hex << std::setw(2) << std::setfill('0') << (int)hash[i];
    }
    return "tag_" + ss.str().substr(0, 16);
}

std::string HyperspaceClient::hash_metadata_value(const std::string& value, const std::vector<uint8_t>& hmac_key) {
    uint8_t hash[EVP_MAX_MD_SIZE];
    unsigned int hash_len = 0;
    HMAC(EVP_sha256(), hmac_key.data(), hmac_key.size(), reinterpret_cast<const unsigned char*>(value.data()), value.size(), hash, &hash_len);

    std::stringstream ss;
    for (unsigned int i = 0; i < hash_len; ++i) {
        ss << std::hex << std::setw(2) << std::setfill('0') << (int)hash[i];
    }
    return "val_" + ss.str();
}

void HyperspaceClient::RegisterCollectionKey(const std::string& collection_name, const std::string& key, const std::string& metric, double noise_sigma, const ::hyperspace::CollectionSchema* schema) {
    collection_keys_[collection_name] = key;
    collection_metrics_[collection_name] = metric;
    collection_noise_sigmas_[collection_name] = noise_sigma;
    if (schema) {
        collection_schemas_[collection_name] = *schema;
    }
    encryption_contexts_.erase(collection_name);
}

std::shared_ptr<HyperspaceClient::EncryptionContext> HyperspaceClient::get_encryption_context(const std::string& collection, size_t vector_dim, const std::string& metric) {
    if (collection.empty()) return nullptr;
    auto key_it = collection_keys_.find(collection);
    if (key_it == collection_keys_.end()) return nullptr;

    // Check schema
    if (collection_schemas_.find(collection) == collection_schemas_.end()) {
        CollectionStats stats;
        if (GetCollectionStats(collection, stats)) {
            collection_schemas_[collection] = stats.schema;
        }
    }

    auto ctx_it = encryption_contexts_.find(collection);
    if (ctx_it == encryption_contexts_.end()) {
        auto pair = derive_keys(key_it->second, collection);
        auto ctx = std::make_shared<EncryptionContext>();
        ctx->aes_key = pair.first;
        ctx->hmac_key = pair.second;
        encryption_contexts_[collection] = ctx;
        ctx_it = encryption_contexts_.find(collection);
    }

    auto context = ctx_it->second;

    if (vector_dim > 0) {
        std::string cache_key = std::to_string(vector_dim);
        if (context->projection_matrices.find(cache_key) == context->projection_matrices.end()) {
            bool is_lorentz = (metric == "lorentz" || metric == "poincare");
            size_t matrix_dim = (metric == "poincare") ? vector_dim + 1 : vector_dim;
            if (is_lorentz) {
                context->projection_matrices[cache_key] = math::generate_lorentz_matrix(matrix_dim, context->hmac_key);
            } else {
                context->projection_matrices[cache_key] = math::generate_orthogonal_matrix(matrix_dim, context->hmac_key);
            }
        }
    }

    return context;
}

std::vector<double> HyperspaceClient::project_single_block(const std::vector<double>& sub_vec, const std::string& metric, const EncryptionContext& context, const std::string& block_id) {
    size_t dim = sub_vec.size();
    if (dim == 0) return {};

    std::string cache_key = block_id.empty() ? std::to_string(dim) : (std::to_string(dim) + "_" + block_id);

    if (context.projection_matrices.find(cache_key) == context.projection_matrices.end()) {
        bool is_lorentz = (metric == "lorentz" || metric == "poincare");
        size_t matrix_dim = (metric == "poincare") ? dim + 1 : dim;

        std::vector<uint8_t> seed = context.hmac_key;
        if (!block_id.empty()) {
            std::vector<uint8_t> bid_bytes(block_id.begin(), block_id.end());
            seed.insert(seed.end(), bid_bytes.begin(), bid_bytes.end());
            seed = math::sha256_hash(seed);
        }

        if (is_lorentz) {
            context.projection_matrices[cache_key] = math::generate_lorentz_matrix(matrix_dim, seed);
        } else {
            context.projection_matrices[cache_key] = math::generate_orthogonal_matrix(matrix_dim, seed);
        }
    }

    const auto& matrix = context.projection_matrices[cache_key];

    if (metric == "poincare") {
        auto lorentz = math::poincare_to_lorentz(sub_vec);
        auto proj_lorentz = math::project_vector(lorentz, matrix);
        return math::lorentz_to_poincare(proj_lorentz);
    } else {
        return math::project_vector(sub_vec, matrix);
    }
}

std::vector<double> HyperspaceClient::project_collection_vector(const std::string& collection, const std::vector<double>& vector, const EncryptionContext& context, const std::string& metric) {
    auto schema_it = collection_schemas_.find(collection);
    if (schema_it == collection_schemas_.end() || schema_it->second.components_size() == 0) {
        return project_single_block(vector, metric, context);
    }

    const auto& schema = schema_it->second;

    std::unordered_map<std::string, std::vector<size_t>> component_cutoffs;
    for (int i = 0; i < schema.cascade_pipeline_size(); ++i) {
        const auto& layer = schema.cascade_pipeline(i);
        if (!layer.component_name().empty() && layer.cutoff_dimension() > 0) {
            component_cutoffs[layer.component_name()].push_back(layer.cutoff_dimension());
        }
    }

    for (auto& [name, cutoffs] : component_cutoffs) {
        std::sort(cutoffs.begin(), cutoffs.end());
        cutoffs.erase(std::unique(cutoffs.begin(), cutoffs.end()), cutoffs.end());
    }

    std::vector<double> projected_parts;
    size_t current_offset = 0;

    for (int i = 0; i < schema.components_size(); ++i) {
        const auto& comp = schema.components(i);
        std::string comp_name = comp.name();
        std::string comp_metric = comp.metric();
        size_t comp_dim = comp.full_dimension();

        if (current_offset >= vector.size()) {
            break;
        }

        size_t end = current_offset + comp_dim;
        if (end > vector.size()) {
            end = vector.size();
        }

        std::vector<double> sub_vec(vector.begin() + current_offset, vector.begin() + end);
        if (sub_vec.size() < comp_dim) {
            sub_vec.resize(comp_dim, 0.0);
        }

        const auto& cutoffs = component_cutoffs[comp_name];
        std::vector<size_t> valid_cutoffs;
        for (size_t c : cutoffs) {
            if (c < comp_dim) {
                valid_cutoffs.push_back(c);
            }
        }

        std::vector<double> proj_sub;
        if (valid_cutoffs.empty()) {
            proj_sub = project_single_block(sub_vec, comp_metric, context);
        } else {
            size_t block_start = 0;
            for (size_t cutoff : valid_cutoffs) {
                std::vector<double> block_data(sub_vec.begin() + block_start, sub_vec.begin() + cutoff);
                std::string bid = comp_name + "_block_" + std::to_string(block_start) + "_" + std::to_string(cutoff);
                auto proj_block = project_single_block(block_data, comp_metric, context, bid);
                proj_sub.insert(proj_sub.end(), proj_block.begin(), proj_block.end());
                block_start = cutoff;
            }
            if (block_start < comp_dim) {
                std::vector<double> block_data(sub_vec.begin() + block_start, sub_vec.end());
                std::string bid = comp_name + "_block_" + std::to_string(block_start) + "_" + std::to_string(comp_dim);
                auto proj_block = project_single_block(block_data, comp_metric, context, bid);
                proj_sub.insert(proj_sub.end(), proj_block.begin(), proj_block.end());
            }
        }

        projected_parts.insert(projected_parts.end(), proj_sub.begin(), proj_sub.end());
        current_offset += comp_dim;
    }

    if (current_offset < vector.size()) {
        projected_parts.insert(projected_parts.end(), vector.begin() + current_offset, vector.end());
    }

    return projected_parts;
}

std::vector<::hyperspace::Filter> HyperspaceClient::encrypt_filters(const std::vector<::hyperspace::Filter>& filters, const EncryptionContext& context) {
    std::vector<::hyperspace::Filter> res;
    res.reserve(filters.size());

    for (const auto& f : filters) {
        ::hyperspace::Filter nf;
        if (f.has_match()) {
            auto* m = nf.mutable_match();
            m->set_key(hash_metadata_key(f.match().key(), context.hmac_key));
            m->set_value(hash_metadata_value(f.match().value(), context.hmac_key));
        } else if (f.has_prefix()) {
            auto* p = nf.mutable_prefix();
            p->set_key(hash_metadata_key(f.prefix().key(), context.hmac_key));
            p->set_prefix(hash_metadata_value(f.prefix().prefix(), context.hmac_key));
        } else if (f.has_and_op()) {
            auto* a = nf.mutable_and_op();
            std::vector<::hyperspace::Filter> sub_filters(f.and_op().conditions().begin(), f.and_op().conditions().end());
            auto enc_sub = encrypt_filters(sub_filters, context);
            for (const auto& sf : enc_sub) {
                *a->add_conditions() = sf;
            }
        } else if (f.has_or_op()) {
            auto* o = nf.mutable_or_op();
            std::vector<::hyperspace::Filter> sub_filters(f.or_op().conditions().begin(), f.or_op().conditions().end());
            auto enc_sub = encrypt_filters(sub_filters, context);
            for (const auto& sf : enc_sub) {
                *o->add_conditions() = sf;
            }
        } else if (f.has_not_op()) {
            auto* n = nf.mutable_not_op();
            auto enc_sub = encrypt_filters({f.not_op().condition()}, context);
            *n->mutable_condition() = enc_sub[0];
        } else {
            nf = f;
        }
        res.push_back(nf);
    }
    return res;
}

bool HyperspaceClient::CreateCollectionSecure(const std::string& name, const ::hyperspace::CollectionSchema& schema, const std::string& encryption_key, double noise_sigma) {
    std::string metric = "l2";
    if (schema.components_size() > 0) {
        metric = schema.components(0).metric();
    }
    if (!encryption_key.empty()) {
        RegisterCollectionKey(name, encryption_key, metric, noise_sigma, &schema);
    }
    return CreateCollection(name, schema);
}

} // namespace hyperspace
