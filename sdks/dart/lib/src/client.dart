import 'dart:async';
import 'package:grpc/grpc.dart';
// Note: In an actual project, run `./generate_protos.sh` and import generated files.
// import 'generated/hyperspace.pbgrpc.dart';

class HyperspaceClient {
  final ClientChannel _channel;
  // late final DatabaseClient _stub; // Placeholder generated stub
  final String apiKey;
  final String? tenantId;

  HyperspaceClient(String address, int port, {this.apiKey = '', this.tenantId})
      : _channel = ClientChannel(
          address,
          port: port,
          options: const ChannelOptions(credentials: ChannelCredentials.insecure()),
        ) {
    // _stub = DatabaseClient(_channel, options: _callOptions());
  }

  CallOptions _callOptions() {
    final metadata = <String, String>{
      if (apiKey.isNotEmpty) 'authorization': 'Bearer $apiKey',
      if (tenantId != null) 'x-hyperspace-user-id': tenantId!,
    };
    return CallOptions(metadata: metadata);
  }

  Future<void> close() async {
    await _channel.shutdown();
  }

  // Example API Signature
  Future<bool> createCollection(String name, int dimension, String metric) async {
    // return _stub.createCollection(...);
    return true; 
  }

  Future<bool> insert(int id, List<double> vector, {String collection = ''}) async {
    // return _stub.insert(...);
    return true;
  }
}
