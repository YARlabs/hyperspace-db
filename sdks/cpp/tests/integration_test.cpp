// Integration test against a running server (>= 4.2.5):
//   HS_TEST_ENDPOINT=localhost:50051 ./hyperspace_sdk_integration_test
// Covers per-request ef_search and compact (f32) vectors end to end.
#include "hyperspace/client.hpp"

#include <cstdlib>
#include <iostream>
#include <string>

namespace {
int failures = 0;
void check(bool ok, const std::string& what) {
    std::cout << (ok ? "ok    " : "FAIL  ") << what << "\n";
    if (!ok) ++failures;
}
}  // namespace

int main() {
    const char* ep = std::getenv("HS_TEST_ENDPOINT");
    const char* key = std::getenv("HS_TEST_API_KEY");
    hyperspace::HyperspaceClient client(ep ? ep : "localhost:50051", key ? key : "I_LOVE_HYPERSPACEDB");

    const std::string col = "cpp_sdk_it";
    client.DeleteCollection(col);
    ::hyperspace::CollectionSchema schema;
    auto* comp = schema.add_components();
    comp->set_name("main");
    comp->set_metric("l2");
    comp->set_full_dimension(4);
    comp->set_weight(1.0f);
    check(client.CreateCollection(col, schema), "create collection");

    for (uint32_t i = 0; i < 50; ++i) {
        double x = static_cast<double>(i) / 50.0;
        client.Insert(i, {x, x, x, x}, col);
    }

    auto base = client.Search({0.5, 0.5, 0.5, 0.5}, 3, col);
    check(!base.empty() && base.front().id == 25, "search finds the exact vector");

    auto wide = client.Search({0.5, 0.5, 0.5, 0.5}, 3, col, "", 0.0f, nullptr, 0, false, false, {}, false,
                              std::nullopt, /*ef_search=*/256);
    check(!wide.empty() && wide.front().id == 25, "search with ef_search=256");

    auto batch = client.SearchBatch({{0.1, 0.1, 0.1, 0.1}, {0.9, 0.9, 0.9, 0.9}}, 1, col, 128);
    check(batch.size() == 2 && !batch[0].empty() && batch[0][0].id == 5, "batch search with ef_search");

    auto multi = client.SearchMultiCollection({col}, {0.2, 0.2, 0.2, 0.2}, 2, 64);
    check(multi.count(col) == 1 && !multi[col].empty() && multi[col][0].id == 10,
          "multi-collection search with ef_search");

    client.SetCompactVectors(true);
    auto compact = client.Search({0.5, 0.5, 0.5, 0.5}, 1, col);
    check(!compact.empty() && compact.front().id == 25, "compact f32 vectors");

    check(client.DeleteCollection(col), "delete collection");
    std::cout << (failures == 0 ? "ALL PASSED" : "FAILED") << "\n";
    return failures == 0 ? 0 : 1;
}
