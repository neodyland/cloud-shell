import websockets
import asyncio
import json


async def hello():
    async with websockets.connect("ws://localhost:8000") as ws:
        print(await ws.recv())
        ready = False
        data = json.loads(await ws.recv())
        while ws.open:
            data = json.loads(await ws.recv())
            if data["op"] == 2:
                print("Ready")
                ready = True
                await ws.send(json.dumps({
                    "op": 3,
                    "data": "help\n"
                }))
            elif data["op"] == 4:
                print(data["data"])


asyncio.run(hello())