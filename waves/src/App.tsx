import React, { useEffect, useState } from "react";
import { IMessageEvent, w3cwebsocket as W3CWebSocket } from "websocket";
import "./App.css";
import { hello } from "./wasmProxy";

function App() {
    const [welcome, setWelcome] = useState<String>("Not welcome yet");
    const [rate, setRate] = useState<String>("No rate received yet");

    useEffect(() => {
        hello("World").then((result) => {
            setWelcome(result);
        });
    }, []);

    useEffect(() => {
        const client = new W3CWebSocket("ws://127.0.0.1:3030/rate");
        client.onopen = () => {
            console.log("WebSocket Client Connected");
        };
        client.onmessage = (rate: IMessageEvent) => {
            setRate(rate.data as string);
        };
    }, []);

    return (
        <div className="App">
            <header className="App-header">
                <div>Rust lib says: `{welcome}`</div>
                <div>Your exchange rate is: `{rate}`</div>
            </header>
        </div>
    );
}

export default App;
