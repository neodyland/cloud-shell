import websockets
import asyncio
import json


async def hello():
    loop = asyncio.get_running_loop()
    async with websockets.connect("ws://localhost:8000") as ws:
        print(await ws.recv())
        ready = False
        while ws.open:
            data = json.loads(await ws.recv())
            if data["op"] == 2:
                print("Ready")
                await ws.send(json.dumps({
                    "op": 3,
                    "data": "pacman -Syyu --noconfirm neofetch && neofetch\n"
                }))
            if data["op"] == 4:
                print(data["data"])


asyncio.run(hello())