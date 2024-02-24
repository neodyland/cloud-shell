import websockets
import asyncio


async def hello():
    async with websockets.connect("ws://localhost:8000") as ws:
        print(await ws.recv())


asyncio.run(hello())