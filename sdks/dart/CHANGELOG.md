## 4.2.5

- `compactVectors` client option: send vectors as `repeated float` (`vector_f32`), half the
  wire size of `repeated double` (requires server >= 4.2.5).
- Protobuf stubs regenerated with `protoc_plugin` 25 (`protobuf` 6): `vector_f32` on `SearchRequest`,
  `InsertRequest`, `VectorData`, `SearchMultiCollectionRequest`, plus the PDK extension messages
  (`ExtensionRequest`, `ExtensionInfo`, `ListExtensionsResponse`, ...) that the 4.2.4 stubs lacked.
- Integration tests read `HS_TEST_HOST`, `HS_TEST_PORT`, `HS_TEST_API_KEY` and `HS_TEST_COMPACT`.
- New quantization levels accepted by `createCollection(quantization: ...)`: `rabitq`, `pq4`.

## 4.2.4

- Release version 4.2.4

## 4.2.3

- Release version 4.2.3

## 4.2.0

- Release version 4.2.0

## 4.1.0

- Release version 4.1.0

## 4.0.0

- Release version 4.0.0

## 3.1.8

- Release version 3.1.8

## 3.1.7

- Release version 3.1.7

## 3.1.6

- Release version 3.1.6

## 3.1.5

- Release version 3.1.5

## 3.1.4

- Release version 3.1.4

## 3.1.3

- Release version 3.1.3

## 3.1.2

- Official Dart/Flutter SDK release for HyperspaceDB.
