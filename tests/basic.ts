import WebSocket from "ws";

const ws = new WebSocket("ws://localhsot:8000/ws");

ws.on('error', console.error);

ws.on('open', function open() {
  ws.send('something');
});

ws.on('message', function message(data) {
  const payload = JSON.parse(data);
  if (payload.t === "Ready") {
    ws.send(JSON.stringify({
      t: "Stdin",
      c: "pacman -Syyu --noconfirm",
    }));
  } else if (payload.t === "Stdout" || payload.t === "Stderr") {
    console.log(payload.c);
  };
});
