import React, { useState } from "react";
import "./App.css";

function App() {
    const [welcome, setWelcome] = useState<String>("Not welcome yet");
    const [rate, setRate] = useState<String>("No rate received yet");

    import("./wallet-lib/pkg").then(({ hello }) => {
        let welcome = hello("World");
        setWelcome(welcome);
    });

    fetch("http://localhost:3030/rate")
        .then((res) => res.json())
        .then((result) => {
            setRate(result);
        }).catch((_error) => {
            console.log("Could not receive rate from bobtimus");
    });

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
