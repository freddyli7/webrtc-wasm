import init, {run_webrtc} from './pkg/webrtc_wasm.js';

async function main() {
    await init();
    document.getElementById('startBtn').onclick = () => {
        run_webrtc()
            .then(sdp => {
                console.log("SDP from WASM:", sdp);
                const sdpString = JSON.stringify({sdp: sdp.sdp, type: sdp.type});
                console.log("SDP base64:", btoa(sdpString));
            })
            .catch(e => {
                console.error("run_webrtc() error:", e);
            });
    };
}

main();
