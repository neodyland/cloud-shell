import websockets
import asyncio
import json


async def hello():
    loop = asyncio.get_running_loop()
    async with websockets.connect("ws://localhost:8000/ws") as ws:
        print(await ws.recv())
        ready = False
        while ws.open:
            data = json.loads(await ws.recv())
            if data["t"] == "Hello":
                print("Ready")
                await ws.send(
                    json.dumps({"t": "Stdin", "c": "uname -r && env && df -h\n"})
                )
            elif data["t"] == "Stdout" or data["t"] == "Stderr":
                print(data["c"])


asyncio.run(hello())
