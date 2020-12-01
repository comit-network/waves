import React, { useState } from "react";
import "./App.css";

function App() {
    import("./native/pkg").then(({ hello }) => {
        let welcome = hello("World");
        setWelcome(welcome);
    });
    const [welcome, setWelcome] = useState<String>("Not welcome yet");

    return (
        <div className="App">
            <header className="App-header">
                <div>Rust lib says: `{welcome}`</div>
            </header>
        </div>
    );
}

export default App;
