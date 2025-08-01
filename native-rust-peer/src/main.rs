use std::io::{self};
use std::sync::Arc;

use serde_json;
use webrtc::api::APIBuilder;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::ice_transport::ice_gathering_state::RTCIceGatheringState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = APIBuilder::new().build();
    let config = RTCConfiguration::default();

    let config = RTCConfiguration {
        ice_servers: vec![
            RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let peer_connection = api.new_peer_connection(config).await?;

    // Set handler for data channel
    peer_connection.on_data_channel(Box::new(|dc: Arc<RTCDataChannel>| {
        println!("DataChannel '{}' opened", dc.label());
        Box::pin(async move {
            let dc2 = Arc::clone(&dc);
            dc.on_open(Box::new(move || {
                let dc3 = Arc::clone(&dc2);
                Box::pin(async move {
                    println!("Rust: sending Hello Browser");
                    let _ = dc3.send_text("Hello Browser".to_string()).await;
                })
            }));

            dc.on_message(Box::new(move |msg| {
                println!("Rust received: {}", String::from_utf8_lossy(&msg.data));
                Box::pin(async {})
            }));
        })
    }));

    peer_connection.on_ice_connection_state_change(Box::new(|state| {
        println!("Rust ICE connection state: {}", state);
        Box::pin(async {})
    }));

    // === STEP 1: Paste offer from browser ===
    // println!("Paste browser's SDP offer JSON:");
    // let mut sdp_offer = String::new();
    // io::stdin().read_line(&mut sdp_offer)?;
    // let offer: RTCSessionDescription = serde_json::from_str(sdp_offer.trim())?;
    // peer_connection.set_remote_description(offer).await?;

    println!("Loading browser's SDP offer JSON from file offer.b64:");
    let base64_str = std::fs::read_to_string("./offer.b64")?;
    let json = String::from_utf8(base64::decode(base64_str.trim())?)?;
    let offer: RTCSessionDescription = serde_json::from_str(&json)?;
    peer_connection.set_remote_description(offer).await?;

    // === STEP 2: Create and print answer ===
    let answer = peer_connection.create_answer(None).await?;
    peer_connection.set_local_description(answer.clone()).await?;


    // Wait for ICE gathering to complete
    while peer_connection.ice_gathering_state() != RTCIceGatheringState::Complete {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    // Now fetch the complete SDP with gathered ICE candidates
    let final_answer = peer_connection.local_description().await.unwrap();
    println!("\nCopy this SDP answer to browser:\n{}", serde_json::to_string(&final_answer)?);

    // Keep the process alive
    tokio::signal::ctrl_c().await?;
    Ok(())
}
