


wtest:
	@grpcurl -plaintext -import-path ./esto/proto -proto esto_rpc.proto -d '{"entity_id": "95C6D7EF-58E1-4C32-A1C6-7A11CB63C759", "entity_type": "MyThing", "event_name": "ThingDone", "event_data": "{}" }' localhost:50051 esto_rpc.Esto/StoreRecord

rtest:
	@grpcurl -plaintext -import-path ./esto/proto -proto esto_rpc.proto -d '{"entity_id": "95C6D7EF-58E1-4C32-A1C6-7A11CB63C759"}' localhost:50051 esto_rpc.Esto/ReadRecord

pyclient:
	python -m grpc_tools.protoc -Iesto/proto --python_out=test/pyclient --grpc_python_out=test/pyclient esto/proto/esto_rpc.proto
