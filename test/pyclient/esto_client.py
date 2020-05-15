import json
import time

import grpc
from pyclient.esto_rpc_pb2_grpc import EstoStub
from pyclient.esto_rpc_pb2 import StoreRequest, ReadRequest, ReadEventList, Event

class Client:
    def __init__(self):
        channel = grpc.insecure_channel('localhost:50051')
        self.stub = EstoStub(channel)

    def write(self, **kwargs):
        arguments = kwargs
        arguments["event_data"] = json.dumps(arguments["event_data"])
        request = StoreRequest(**kwargs)
        reply = self.stub.StoreRecord(request)
        return reply.message

    def read(self, entity_id):
        request = ReadRequest(entity_id=entity_id)
        event_list = self.stub.ReadRecord(request)
        yield from event_list.events


client = Client()

start_time = time.time()
for _ in range(1, 500):
    client.write(
        entity_id="95C6D7EF-58E1-4C32-A1C6-7A11CB63C759",
        entity_type="MyThing",
        event_name="ThingDoneToMyThing",
        event_data={
            "a": 1,
            "b": 2
        }
    )
    # client.read(entity_id="95C6D7EF-58E1-4C32-A1C6-7A11CB63C759")
print(time.time() - start_time)
# y = Client().read(entity_id="95C6D7EF-58E1-4C32-A1C6-7A11CB63C759")
# print(list(y))