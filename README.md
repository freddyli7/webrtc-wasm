1. Compile WASM peer: `wasm-pack build --target web`
2. Run browser side  with `python3 -m http.server 8080` and open localhost:8080 
3. Create the offer from the browser 
4. Open the browser console and copy the base64 string of the SDP offer 
5. Save the string into offer.b64 file 
6. Run the native rust peer with `Cargo run` under `/native-rust-peer`
7. Copy the SDP answer from rust peer the paste it into browser side.