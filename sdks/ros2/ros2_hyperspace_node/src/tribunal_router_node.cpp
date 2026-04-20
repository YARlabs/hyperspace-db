#include <rclcpp/rclcpp.hpp>
#include <hyperspace_interfaces/srv/navigate_to_attractor.hpp>
#include <hyperspace_interfaces/srv/insert_text.hpp>
#include <hyperspace_interfaces/srv/search_text.hpp>
#include <hyperspace_interfaces/srv/vectorize.hpp>
#include <hyperspace_interfaces/srv/delete.hpp>
#include <hyperspace_interfaces/msg/search_result.hpp>

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
        RCLCPP_INFO(this->get_logger(), "Inserting Text [ID: %d] into [%s]", request->id, request->collection.c_str());
        bool ok = client_->InsertText(request->id, request->text, request->collection);
        response->success = ok;
        response->message = ok ? "Successfully inserted." : "Failed to insert into HyperspaceDB.";
    }

    void handle_search_text(
        const std::shared_ptr<hyperspace_interfaces::srv::SearchText::Request> request,
        std::shared_ptr<hyperspace_interfaces::srv::SearchText::Response> response)
    {
        RCLCPP_INFO(this->get_logger(), "Searching for '%s' in [%s] (Alpha: %f)", request->query.c_str(), request->collection.c_str(), request->hybrid_alpha);
        auto results = client_->SearchText(request->query, request->top_k, request->collection, request->hybrid_alpha);
        
        for (const auto& r : results) {
            hyperspace_interfaces::msg::SearchResult msg;
            msg.id = r.id;
            msg.distance = r.score;
            for (const auto& [k, v] : r.metadata) {
                msg.metadata_keys.push_back(k);
                msg.metadata_values.push_back(v);
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

    std::unique_ptr<hyperspace::HyperspaceClient> client_;
    
    rclcpp::Service<hyperspace_interfaces::srv::NavigateToAttractor>::SharedPtr navigate_service_;
    rclcpp::Service<hyperspace_interfaces::srv::InsertText>::SharedPtr insert_service_;
    rclcpp::Service<hyperspace_interfaces::srv::SearchText>::SharedPtr search_service_;
    rclcpp::Service<hyperspace_interfaces::srv::Vectorize>::SharedPtr vectorize_service_;
    rclcpp::Service<hyperspace_interfaces::srv::Delete>::SharedPtr delete_service_;
};

int main(int argc, char **argv) {
    rclcpp::init(argc, argv);
    rclcpp::spin(std::make_shared<TribunalRouterNode>());
    rclcpp::shutdown();
    return 0;
}
