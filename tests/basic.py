import websockets
import asyncio
import json


async def hello():
    async with websockets.connect("ws://localhost:8000") as ws:
        print(await ws.recv())
        await ws.send("Hello, World!")
        ready = False
        while ws.open:
            data = json.loads(await ws.recv())
            if data["op"] == 2:
                print("Ready")
                ready = True
                await ws.send(json.dumps({
                    "op": 4,
                    "data": "help"
                }))


asyncio.run(hello())