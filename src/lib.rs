use wasm_bindgen::prelude::*;
use wasm_bindgen::JsError;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    RtcPeerConnection, RtcSessionDescriptionInit, MessageEvent, RtcDataChannel, RtcSdpType,
};
use js_sys::{Object, Reflect, Array};
use once_cell::unsync::OnceCell;

thread_local! {
    static PC_GLOBAL: OnceCell<RtcPeerConnection> = OnceCell::new();
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    Ok(())
}

#[wasm_bindgen]
pub async fn run_webrtc() -> Result<JsValue, JsError> {
    web_sys::console::log_1(&"run_webrtc(): started".into());

    // === Build STUN ICE server ===
    let stun_server = JsValue::from_str("stun:stun.l.google.com:19302");
    web_sys::console::log_1(&"run_webrtc(): Building ICE server".into());

    let ice_server = Object::new();
    Reflect::set(&ice_server, &JsValue::from_str("urls"), &stun_server)
        .map_err(log_and_convert)?;

    let ice_servers = Array::new();
    ice_servers.push(&ice_server);

    // === Create PeerConnection ===
    web_sys::console::log_1(&"run_webrtc(): Creating PeerConnection".into());
    let mut config = web_sys::RtcConfiguration::new();
    config.ice_servers(&ice_servers);
    let pc = RtcPeerConnection::new_with_configuration(&config)
        .map_err(log_and_convert)?;

    // === DataChannel ===
    web_sys::console::log_1(&"run_webrtc(): Creating DataChannel".into());
    let dc = pc.create_data_channel("chat");
    let value = dc.clone();

    // Open event
    let on_open = Closure::wrap(Box::new(move || {
        web_sys::console::log_1(&"WASM: DataChannel open".into());
        let _ = value.send_with_str("Hello from WASM");
    }) as Box<dyn FnMut()>);
    dc.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    on_open.forget();

    // Message event
    let on_msg = Closure::wrap(Box::new(move |e: MessageEvent| {
        if let Some(msg) = e.data().as_string() {
            web_sys::console::log_1(&format!("WASM received: {}", msg).into());
        }
    }) as Box<dyn FnMut(_)>);
    dc.set_onmessage(Some(on_msg.as_ref().unchecked_ref()));
    on_msg.forget();

    // === Create offer ===
    web_sys::console::log_1(&"run_webrtc(): Creating offer".into());
    let offer = JsFuture::from(pc.create_offer())
        .await
        .map_err(log_and_convert)?
        .unchecked_into();

    web_sys::console::log_1(&"run_webrtc(): Setting local description".into());
    JsFuture::from(pc.set_local_description(&offer))
        .await
        .map_err(log_and_convert)?;

    // === Wait until ICE is complete ===
    web_sys::console::log_1(&"run_webrtc(): Waiting for ICE gathering".into());
    loop {
        if pc.ice_gathering_state() == web_sys::RtcIceGatheringState::Complete {
            break;
        }
        gloo_timers::future::TimeoutFuture::new(100).await;
    }

    // === Get final SDP ===
    web_sys::console::log_1(&"run_webrtc(): Getting local description".into());
    let sdp = pc
        .local_description()
        .ok_or_else(|| JsError::new("No local description"))?;

    // Return as plain JS object
    web_sys::console::log_1(&format!("run_webrtc(): Returning SDP: {:?}", sdp.sdp()).into());
    let result = Object::new();
    Reflect::set(&result, &JsValue::from_str("sdp"), &JsValue::from_str(&sdp.sdp()))
        .map_err(log_and_convert)?;

    let sdp_type = match sdp.type_() {
        web_sys::RtcSdpType::Offer => "offer",
        web_sys::RtcSdpType::Answer => "answer",
        web_sys::RtcSdpType::Pranswer => "pranswer",
        web_sys::RtcSdpType::Rollback => "rollback",
        _ => "offer", // fallback (safe default)
    };
    Reflect::set(&result, &JsValue::from_str("type"), &JsValue::from_str(&sdp_type))
        .map_err(log_and_convert)?;

    web_sys::console::log_1(&"run_webrtc(): Finished successfully".into());

    PC_GLOBAL.with(|cell| {
        cell.set(pc.clone()).ok();
    });

    Ok(result.into())
}


/// Converts `JsValue` into `JsError`
fn jsvalue_to_jserror(e: JsValue) -> JsError {
    if let Some(s) = e.as_string() {
        JsError::new(&s)
    } else {
        JsError::new("Unknown JS error")
    }
}

fn log_and_convert(e: JsValue) -> JsError {
    web_sys::console::error_1(&e);
    if let Some(s) = e.as_string() {
        JsError::new(&s)
    } else {
        JsError::new("JS error (see console)")
    }
}

#[wasm_bindgen]
pub async fn set_answer(answer_json: String) -> Result<(), JsError> {
    let parsed: serde_json::Value = serde_json::from_str(&answer_json)
        .map_err(|_| JsError::new("Invalid JSON"))?;

    let sdp = parsed["sdp"].as_str().unwrap();
    let mut desc = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
    desc.sdp(sdp);

    PC_GLOBAL.with(|cell| {
        let pc = cell.get().expect("PeerConnection not initialized");
        let fut = JsFuture::from(pc.set_remote_description(&desc));
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = fut.await {
                web_sys::console::error_1(&e);
            }
        });
    });

    Ok(())
}