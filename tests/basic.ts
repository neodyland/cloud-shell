import WebSocket from "ws";

const ws = new WebSocket("ws://localhsot:8000/ws");

ws.on('error', console.error);

ws.on('open', function open() {
  ws.send('something');
});

ws.on('message', function message(data) {
  const payload = JSON.parse(data);
  if (payload.op === 2) {
    ws.send(JSON.stringify({
      op: 3,
      data: "pacman -Syyu --noconfirm",
    });
  } else if (payload.op === 4) {
    console.log(payload.data);
  };
});
