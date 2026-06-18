#include <rclcpp/rclcpp.hpp>
#include <hyperspace_interfaces/srv/navigate_to_attractor.hpp>
#include <hyperspace_interfaces/srv/insert_text.hpp>
#include <hyperspace_interfaces/srv/search_text.hpp>
#include <hyperspace_interfaces/srv/vectorize.hpp>
#include <hyperspace_interfaces/srv/delete.hpp>
#include <hyperspace_interfaces/srv/get_collection_stats.hpp>
#include <hyperspace_interfaces/srv/exists.hpp>
#include <hyperspace_interfaces/srv/update_collection.hpp>
#include <hyperspace_interfaces/srv/create_snapshot.hpp>
#include <hyperspace_interfaces/srv/vacuum.hpp>
#include <hyperspace_interfaces/srv/search_multi_collection.hpp>
#include <hyperspace_interfaces/srv/search.hpp>
#include <hyperspace_interfaces/srv/get_node.hpp>
#include <hyperspace_interfaces/srv/get_neighbors.hpp>
#include <hyperspace_interfaces/srv/rebuild_index.hpp>
#include <hyperspace_interfaces/srv/create_collection.hpp>
#include <hyperspace_interfaces/srv/trigger_reconsolidation.hpp>
#include <google/protobuf/util/json_util.h>
#include <hyperspace_interfaces/msg/search_result.hpp>
#include <hyperspace_interfaces/msg/system_stats.hpp>

#include <hyperspace/client.hpp>
#include <hyperspace/math.hpp>

#include <vector>
#include <cmath>
#include <memory>

class TribunalRouterNode : public rclcpp::Node {
public:
    TribunalRouterNode() : Node("tribunal_router") {
        // Declare parameters
        this->declare_parameter<std::string>("hyperspace_endpoint", "localhost:50051");
        this->declare_parameter<std::string>("api_key", "I_LOVE_HYPERSPACEDB");
        
        std::string endpoint = this->get_parameter("hyperspace_endpoint").as_string();
        std::string api_key = this->get_parameter("api_key").as_string();

        // Initialize Real Hyperspace Client
        client_ = std::make_unique<hyperspace::HyperspaceClient>(endpoint, api_key);

        // Core AI Navigation Service (Math-driven)
        navigate_service_ = this->create_service<hyperspace_interfaces::srv::NavigateToAttractor>(
            "evaluate_claim_and_navigate",
            std::bind(&TribunalRouterNode::handle_navigate, this, std::placeholders::_1, std::placeholders::_2)
        );

        // Standard Database Services
        insert_service_ = this->create_service<hyperspace_interfaces::srv::InsertText>(
            "hyperspace/insert_text",
            std::bind(&TribunalRouterNode::handle_insert_text, this, std::placeholders::_1, std::placeholders::_2)
        );

        search_service_ = this->create_service<hyperspace_interfaces::srv::SearchText>(
            "hyperspace/search_text",
            std::bind(&TribunalRouterNode::handle_search_text, this, std::placeholders::_1, std::placeholders::_2)
        );

        vectorize_service_ = this->create_service<hyperspace_interfaces::srv::Vectorize>(
            "hyperspace/vectorize",
            std::bind(&TribunalRouterNode::handle_vectorize, this, std::placeholders::_1, std::placeholders::_2)
        );

        delete_service_ = this->create_service<hyperspace_interfaces::srv::Delete>(
            "hyperspace/delete",
            std::bind(&TribunalRouterNode::handle_delete, this, std::placeholders::_1, std::placeholders::_2)
        );

        stats_service_ = this->create_service<hyperspace_interfaces::srv::GetCollectionStats>(
            "hyperspace/get_collection_stats",
            std::bind(&TribunalRouterNode::handle_get_collection_stats, this, std::placeholders::_1, std::placeholders::_2)
        );

        exists_service_ = this->create_service<hyperspace_interfaces::srv::Exists>(
            "hyperspace/exists",
            std::bind(&TribunalRouterNode::handle_exists, this, std::placeholders::_1, std::placeholders::_2)
        );

        update_collection_service_ = this->create_service<hyperspace_interfaces::srv::UpdateCollection>(
            "hyperspace/update_collection",
            std::bind(&TribunalRouterNode::handle_update_collection, this, std::placeholders::_1, std::placeholders::_2)
        );

        snapshot_service_ = this->create_service<hyperspace_interfaces::srv::CreateSnapshot>(
            "hyperspace/create_snapshot",
            std::bind(&TribunalRouterNode::handle_create_snapshot, this, std::placeholders::_1, std::placeholders::_2)
        );

        vacuum_service_ = this->create_service<hyperspace_interfaces::srv::Vacuum>(
            "hyperspace/vacuum",
            std::bind(&TribunalRouterNode::handle_vacuum, this, std::placeholders::_1, std::placeholders::_2)
        );

        search_multi_service_ = this->create_service<hyperspace_interfaces::srv::SearchMultiCollection>(
            "hyperspace/search_multi_collection",
            std::bind(&TribunalRouterNode::handle_search_multi, this, std::placeholders::_1, std::placeholders::_2)
        );

        search_vector_service_ = this->create_service<hyperspace_interfaces::srv::Search>(
            "hyperspace/search",
            std::bind(&TribunalRouterNode::handle_search, this, std::placeholders::_1, std::placeholders::_2)
        );

        get_node_service_ = this->create_service<hyperspace_interfaces::srv::GetNode>(
            "hyperspace/get_node",
            std::bind(&TribunalRouterNode::handle_get_node, this, std::placeholders::_1, std::placeholders::_2)
        );

        get_neighbors_service_ = this->create_service<hyperspace_interfaces::srv::GetNeighbors>(
            "hyperspace/get_neighbors",
            std::bind(&TribunalRouterNode::handle_get_neighbors, this, std::placeholders::_1, std::placeholders::_2)
        );

        rebuild_index_service_ = this->create_service<hyperspace_interfaces::srv::RebuildIndex>(
            "hyperspace/rebuild_index",
            std::bind(&TribunalRouterNode::handle_rebuild_index, this, std::placeholders::_1, std::placeholders::_2)
        );

        reconsolidation_service_ = this->create_service<hyperspace_interfaces::srv::TriggerReconsolidation>(
            "hyperspace/trigger_reconsolidation",
            std::bind(&TribunalRouterNode::handle_trigger_reconsolidation, this, std::placeholders::_1, std::placeholders::_2)
        );
        
        create_collection_service_ = this->create_service<hyperspace_interfaces::srv::CreateCollection>(
            "hyperspace/create_collection",
            std::bind(&TribunalRouterNode::handle_create_collection, this, std::placeholders::_1, std::placeholders::_2)
        );

        metrics_pub_ = this->create_publisher<hyperspace_interfaces::msg::SystemStats>("hyperspace/metrics", 10);
        metrics_timer_ = this->create_wall_timer(
            std::chrono::seconds(5),
            std::bind(&TribunalRouterNode::publish_metrics, this)
        );

        RCLCPP_INFO(this->get_logger(), "Hyperspace ROS2 Node [Tribunal Router] v3.5 is ONLINE.");
        RCLCPP_INFO(this->get_logger(), "Connected to HyperspaceDB at: %s", endpoint.c_str());
    }

private:
    void handle_navigate(
        const std::shared_ptr<hyperspace_interfaces::srv::NavigateToAttractor::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::NavigateToAttractor::Response> response) 
    {
        // Use Real Hyperbolic Math for Navigation
        try {
            auto pull_dir = hyperspace::math::log_map(request->current_thought, request->context_centroid, 1.0);
            double distance = std::sqrt(std::max(hyperspace::math::norm_sq(pull_dir), 0.0));
            
            double geometric_trust_score = std::exp(-0.4 * distance);

            RCLCPP_INFO(this->get_logger(), "Navigation Evaluation - Confidence: %f", geometric_trust_score);
            
            response->hallucinating = geometric_trust_score < 0.15;
            response->local_entropy = 1.0 - geometric_trust_score;
            response->lyapunov_stability = -geometric_trust_score;
            response->next_velocity_vector = pull_dir;
        } catch (const std::exception& e) {
            RCLCPP_ERROR(this->get_logger(), "Math error in handle_navigate: %s", e.what());
        }
    }

    void handle_insert_text(
        const std::shared_ptr<hyperspace_interfaces::srv::InsertText::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::InsertText::Response> response)
    {
        RCLCPP_INFO(this->get_logger(), "Inserting Text [ID: %u] into [%s]", request->id, request->collection.c_str());
        bool ok = client_->InsertText(request->id, request->text, request->collection);
        response->success = ok;
        response->message = ok ? "Successfully inserted." : "Failed to insert into HyperspaceDB.";
    }

    void handle_search_text(
        const std::shared_ptr<hyperspace_interfaces::srv::SearchText::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::SearchText::Response> response)
    {
        RCLCPP_INFO(this->get_logger(), "Searching for '%s' in [%s] (Alpha: %f, BM25: %s)", 
                   request->query.c_str(), request->collection.c_str(), request->hybrid_alpha, request->bm25_method.c_str());
        
        hyperspace::Bm25Options bm25;
        if (!request->bm25_method.empty()) bm25.method = request->bm25_method;
        if (!request->bm25_language.empty()) bm25.language = request->bm25_language;

        // In a real implementation we would parse filters_json here.
        // For now, passing standard search parameters.
        auto results = client_->SearchText(request->query, request->top_k, request->collection, request->hybrid_alpha, &bm25);
        
        for (const auto& r : results) {
            hyperspace_interfaces::msg::SearchResult msg;
            msg.id = r.id;
            msg.distance = r.score;
            for (auto const& it : r.metadata) {
                msg.metadata_keys.push_back(it.first);
                msg.metadata_values.push_back(it.second);
            }
            response->results.push_back(msg);
        }
    }

    void handle_vectorize(
        const std::shared_ptr<hyperspace_interfaces::srv::Vectorize::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::Vectorize::Response> response)
    {
        auto vec = client_->Vectorize(request->text, request->metric);
        response->vector = vec;
        response->success = !vec.empty();
    }

    void handle_delete(
        const std::shared_ptr<hyperspace_interfaces::srv::Delete::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::Delete::Response> response)
    {
        RCLCPP_INFO(this->get_logger(), "Deleting ID: %d from [%s]", request->id, request->collection.c_str());
        bool ok = client_->Delete(request->id, request->collection);
        response->success = ok;
        response->message = ok ? "Deleted successfully." : "Deletion failed.";
    }

    void handle_get_collection_stats(
        const std::shared_ptr<hyperspace_interfaces::srv::GetCollectionStats::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::GetCollectionStats::Response> response)
    {
        hyperspace::CollectionStats stats;
        bool ok = client_->GetCollectionStats(request->collection, stats);
        response->success = ok;
        if (ok) {
            response->count = stats.count;
            response->indexing_queue = stats.indexing_queue;
        } else {
            response->message = "Failed to get stats";
        }
    }

    void handle_exists(
        const std::shared_ptr<hyperspace_interfaces::srv::Exists::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::Exists::Response> response)
    {
        response->exists = client_->Exists(request->collection);
    }

    void handle_update_collection(
        const std::shared_ptr<hyperspace_interfaces::srv::UpdateCollection::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::UpdateCollection::Response> response)
    {
        bool ok = client_->UpdateCollection(request->collection);
        response->success = ok;
    }

    void handle_create_snapshot(
        const std::shared_ptr<hyperspace_interfaces::srv::CreateSnapshot::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::CreateSnapshot::Response> response)
    {
        (void)request;
        response->success = client_->CreateSnapshot();
    }

    void handle_vacuum(
        const std::shared_ptr<hyperspace_interfaces::srv::Vacuum::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::Vacuum::Response> response)
    {
        (void)request;
        response->success = client_->Vacuum();
    }

    void handle_search_multi(
        const std::shared_ptr<hyperspace_interfaces::srv::SearchMultiCollection::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::SearchMultiCollection::Response> response)
    {
        std::vector<std::string> cols;
        for (const auto& c : request->collections) cols.push_back(c);
        std::vector<double> vec;
        for (double v : request->vector) vec.push_back(v);

        auto results = client_->SearchMultiCollection(cols, vec, request->top_k);
        for (const auto& kv : results) {
            for (const auto& r : kv.second) {
                hyperspace_interfaces::msg::SearchResult msg;
                msg.id = r.id;
                msg.distance = r.score;
                for (auto const& it : r.metadata) {
                    msg.metadata_keys.push_back(it.first);
                    msg.metadata_values.push_back(it.second);
                }
                response->collection_names.push_back(kv.first);
                response->results.push_back(msg);
            }
        }
    }

    void handle_search(
        const std::shared_ptr<hyperspace_interfaces::srv::Search::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::Search::Response> response)
    {
        std::vector<double> vec;
        for (double v : request->vector) vec.push_back(v);

        std::unordered_map<std::string, float> weights;
        for (size_t i = 0; i < request->component_weights_keys.size() && i < request->component_weights_values.size(); ++i) {
            weights[request->component_weights_keys[i]] = request->component_weights_values[i];
        }

        std::optional<float> restart_factor = std::nullopt;
        if (request->restart_factor != 0.0f) {
            restart_factor = request->restart_factor;
        }

        auto results = client_->Search(vec, request->top_k, request->collection, request->hybrid_query, request->hybrid_alpha, nullptr, request->mrl_dimension, request->use_wasserstein, false, weights, request->use_wave, restart_factor);
        for (const auto& r : results) {
            hyperspace_interfaces::msg::SearchResult msg;
            msg.id = r.id;
            msg.distance = r.score;
            for (auto const& it : r.metadata) {
                msg.metadata_keys.push_back(it.first);
                msg.metadata_values.push_back(it.second);
            }
            response->results.push_back(msg);
        }
        response->success = true;
    }

    void handle_get_node(
        const std::shared_ptr<hyperspace_interfaces::srv::GetNode::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::GetNode::Response> response)
    {
        hyperspace::GraphNode node;
        bool ok = client_->GetNode(request->collection, request->id, request->layer, node);
        response->success = ok;
        if (ok) {
            response->id = node.id;
            response->layer = node.layer;
            for (uint32_t n : node.neighbors) response->neighbors.push_back(n);
            // Simulating metadata_json for now
            response->metadata_json = "{}";
        }
    }

    void handle_get_neighbors(
        const std::shared_ptr<hyperspace_interfaces::srv::GetNeighbors::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::GetNeighbors::Response> response)
    {
        hyperspace::GetNeighborsResponse res;
        bool ok = client_->GetNeighbors(request->collection, request->id, request->layer, request->limit, request->offset, res);
        response->success = ok;
        if (ok) {
            for (const auto& n : res.neighbors) response->ids.push_back(n.id);
        }
    }

    void handle_rebuild_index(
        const std::shared_ptr<hyperspace_interfaces::srv::RebuildIndex::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::RebuildIndex::Response> response)
    {
        response->success = client_->RebuildIndex(request->name);
    }

    void handle_trigger_reconsolidation(
        const std::shared_ptr<hyperspace_interfaces::srv::TriggerReconsolidation::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::TriggerReconsolidation::Response> response)
    {
        std::vector<double> target;
        for (double v : request->target_vector) target.push_back(v);
        response->success = client_->TriggerReconsolidation(request->collection, target, request->learning_rate);
    }

    void handle_create_collection(
        const std::shared_ptr<hyperspace_interfaces::srv::CreateCollection::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::CreateCollection::Response> response)
    {
        RCLCPP_INFO(this->get_logger(), "Creating Collection [%s] with Schema JSON", request->name.c_str());
        ::hyperspace::CollectionSchema schema;
        auto status = google::protobuf::util::JsonStringToMessage(request->schema_json, &schema);
        if (!status.ok()) {
            response->success = false;
            response->message = "Invalid Schema JSON: " + status.ToString();
            return;
        }
        bool ok = client_->CreateCollection(request->name, schema);
        response->success = ok;
        response->message = ok ? "Successfully created." : "Failed to create collection.";
    }

    void publish_metrics() {
        // Monitor stream is tricky to bridge to topic directly in one call, 
        // normally we would keep the stream open. For simplicity, we just use the stats we have.
        hyperspace_interfaces::msg::SystemStats msg;
        // In a real implementation, we would pull this from client_->Monitor()
        // Here we just provide placeholder/partial data for parity
        msg.total_collections = 1; 
        msg.total_vectors = 0;
        metrics_pub_->publish(msg);
    }

    std::unique_ptr<hyperspace::HyperspaceClient> client_;
    
    rclcpp::Service<hyperspace_interfaces::srv::NavigateToAttractor>::SharedPtr navigate_service_;
    rclcpp::Service<hyperspace_interfaces::srv::InsertText>::SharedPtr insert_service_;
    rclcpp::Service<hyperspace_interfaces::srv::SearchText>::SharedPtr search_service_;
    rclcpp::Service<hyperspace_interfaces::srv::Search>::SharedPtr search_vector_service_;
    rclcpp::Service<hyperspace_interfaces::srv::Vectorize>::SharedPtr vectorize_service_;
    rclcpp::Service<hyperspace_interfaces::srv::Delete>::SharedPtr delete_service_;
    rclcpp::Service<hyperspace_interfaces::srv::GetCollectionStats>::SharedPtr stats_service_;
    rclcpp::Service<hyperspace_interfaces::srv::Exists>::SharedPtr exists_service_;
    rclcpp::Service<hyperspace_interfaces::srv::UpdateCollection>::SharedPtr update_collection_service_;
    rclcpp::Service<hyperspace_interfaces::srv::CreateSnapshot>::SharedPtr snapshot_service_;
    rclcpp::Service<hyperspace_interfaces::srv::Vacuum>::SharedPtr vacuum_service_;
    rclcpp::Service<hyperspace_interfaces::srv::SearchMultiCollection>::SharedPtr search_multi_service_;
    rclcpp::Service<hyperspace_interfaces::srv::GetNode>::SharedPtr get_node_service_;
    rclcpp::Service<hyperspace_interfaces::srv::GetNeighbors>::SharedPtr get_neighbors_service_;
    rclcpp::Service<hyperspace_interfaces::srv::RebuildIndex>::SharedPtr rebuild_index_service_;
    rclcpp::Service<hyperspace_interfaces::srv::CreateCollection>::SharedPtr create_collection_service_;
    rclcpp::Service<hyperspace_interfaces::srv::TriggerReconsolidation>::SharedPtr reconsolidation_service_;

    rclcpp::Publisher<hyperspace_interfaces::msg::SystemStats>::SharedPtr metrics_pub_;
    rclcpp::TimerBase::SharedPtr metrics_timer_;
};

int main(int argc, char **argv) {
    rclcpp::init(argc, argv);
    rclcpp::spin(std::make_shared<TribunalRouterNode>());
    rclcpp::shutdown();
    return 0;
}
