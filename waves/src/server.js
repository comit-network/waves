const EventEmitter = require("events");
const uuid = require("uuid");
const cors = require("cors");
const express = require("express");

const server = express();
const emitter = new EventEmitter();

server.use(cors());

const state = {
    likes: 19133.74,
    comments: 3,
};

server.get("/rate/lbtc-lusdt", (req, res) => {
    res.writeHead(200, {
        "Content-Type": "text/event-stream",
        "Cache-Control": "no-cache",
        Connection: "keep-alive",
    });

    const listener = (event, data) => {
        console.log(`sending message: ${JSON.stringify(data)}`);
        res.write(`id: ${uuid.v4()}\n`);
        res.write(`event: ${event}\n`);
        res.write(`data: ${JSON.stringify(data)}\n\n`);
    };

    emitter.addListener("push", listener);

    req.on("close", () => {
        emitter.removeListener("push", listener);
    });
});

server.listen(3030, () => {
    console.log("Listen on port 3030...");
});

setInterval(() => {
    state.likes += Math.floor(Math.random() * 10) + 1;

    emitter.emit("push", "rate", {
        ask: state.likes,
        bid: state.likes - 1,
    });
}, 5000);
