import wrap from "it-pb-rpc";
import Libp2p from "libp2p";
import MPLEX from "libp2p-mplex";
import { NOISE } from "libp2p-noise";
import WebSockets from "libp2p-websockets";
import filters from "libp2p-websockets/src/filters";
import { Multiaddr } from "multiaddr";
import PeerId from "peer-id";
import { useEffect, useState } from "react";
import { rendezvous } from "./proto";

const protocols = ["/rendezvous/1.0.0"];

// Currently configure to waves own rendezvous instance for testing
const RENDEZVOUS_NODE_ADDR = "/dns4/rendezvous.coblox.tech/tcp/9961/wss";
const RENDEZVOUS_NODE_PEER_ID = "12D3KooWJwDRoLzZw7wgJR3z7Eb8AhK79ixowPF5FwA74SfnoiNz";

export function getPeerId(): PeerId {
    return PeerId.createFromB58String(RENDEZVOUS_NODE_PEER_ID);
}

export function getMultiAddress(): Multiaddr {
    return new Multiaddr(RENDEZVOUS_NODE_ADDR);
}

const transportKey = WebSockets.prototype[Symbol.toStringTag];

export class Rendezvous {
    private constructor(private libp2p: Libp2p, private peerId: PeerId) {}

    public static async newInstance(): Promise<Rendezvous> {
        let multiaddr = getMultiAddress();
        let peerId = getPeerId();

        const node = await Libp2p.create({
            modules: {
                transport: [WebSockets],
                connEncryption: [NOISE],
                streamMuxer: [MPLEX],
            },
            config: {
                transport: {
                    [transportKey]: {
                        // note: in case connection to IP is needed for dev purposes filter has to be changed to `all`
                        filter: filters.dnsWss,
                    },
                },
            },
        });

        await node.start();
        node.peerStore.addressBook.add(peerId, [multiaddr]);

        return new Rendezvous(node, peerId);
    }

    public async discover(): Promise<rendezvous.pb.Message.IDiscoverResponse | null | undefined> {
        try {
            console.log("dialing...");
            const { stream } = await this.libp2p.dialProtocol(
                this.peerId,
                protocols,
            );
            console.log("dialed");

            let dm = rendezvous.pb.Message.Discover.create({ ns: "blablubb" });
            let msg = rendezvous.pb.Message.create({ type: rendezvous.pb.Message.MessageType.DISCOVER, discover: dm });

            // Note: unable to use readPB because of the requirement to pass in
            // data as Buffer but encode returns Buffer as well which results in double wrapping.
            await wrap(stream).writeLP(Buffer.from(rendezvous.pb.Message.encode(msg).finish()));

            let response = await wrap(stream).readPB({
                decode: bytes => {
                    return rendezvous.pb.Message.decode(bytes);
                },
            });
            let discoverResponse = response.discoverResponse;

            await stream.close();

            console.log(discoverResponse);

            return discoverResponse;
        } catch (e) {
            if (e instanceof Error && e.message.includes("No transport available")) {
                // Since we have set the transport `filters` to `all` so we can use ip-addresses to connect,
                // we can run into the problem that we try to connect on a port that is not configured for
                // websockets if connecting on the websocket address fails. In this case we just log a warning.
                console.warn("skipping port that is not configured for websockets");
            } else {
                throw e;
            }
        }

        throw Error("All attempts to fetch a quote failed.");
    }
}

export default function useRendezvous() {
    let [asb, setAsb] = useState<Rendezvous | null>(null);

    useEffect(() => {
        async function initAsb() {
            try {
                const asb = await Rendezvous.newInstance();
                setAsb(asb);
            } catch (e) {
                console.error(e);
            }
        }

        if (!asb) {
            initAsb();
        }
    }, [asb]);

    return asb;
}
