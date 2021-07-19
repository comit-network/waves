import React, { useEffect, useState } from "react";
import { rendezvous } from "./proto";
import useRendezvous from "./useRendezvous";

function App() {
    const [status, setStatus] = useState<string | null>(null);
    const [discovery, setDiscovery] = useState<rendezvous.pb.Message.IDiscoverResponse | null | undefined>(null);

    const rendezvous = useRendezvous();

    useEffect(() => {
        const interval = setInterval(async () => {
            if (rendezvous) {
                try {
                    const discoverResponse = await rendezvous.discover();
                    setDiscovery(discoverResponse);
                    setStatus("Discovered:");
                } catch (e) {
                    setStatus("Error: " + e.toString());
                }
            } else {
                setStatus("Rendezvous server problem...");
            }
        }, 10000);
        return () => clearInterval(interval);
    }, [rendezvous, discovery]);

    if (discovery) {
        return (
            <div>
                <div>Status: {status}</div>
                <div>
                    {JSON.stringify(discovery)}
                </div>
            </div>
        );
    } else {
        if (status) {
            return <div>Status: {status}</div>;
        }

        return <div>Setting up...</div>;
    }
}

export default App;
