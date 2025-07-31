import init, {run_webrtc, set_answer} from './pkg/webrtc_wasm.js';

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

    document.getElementById('answerBtn').onclick = async () => {
        const answerB64 = prompt("Paste base64 answer from Rust peer:");
        await set_answer(answerB64);
        console.log("Answer set, DataChannel should now open!");
    };
}

main();
