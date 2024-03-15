import websockets
import asyncio
import json


async def runner(ws):
    while ws.open:
        print("Ok")
        data = json.loads(await ws.recv())
        if data["op"] == 4:
            print(data["data"])


async def hello():
    async with websockets.connect("ws://localhost:8000") as ws:
        print(await ws.recv())
        ready = False
        while ws.open:
            data = json.loads(await ws.recv())
            if data["op"] == 2:
                print("Ready")
                break
        asyncio.create_task(runner(ws))
        print("Ok")
        while ws.open:
            cmd = input("Enter command: ")
            await ws.send(json.dumps({
                "op": 3,
                "d": cmd
            }))


asyncio.run(hello())