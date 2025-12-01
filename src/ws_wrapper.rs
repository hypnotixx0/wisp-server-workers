use worker::{
    console_log,
    js_sys::{ArrayBuffer, Uint8Array},
    wasm_bindgen::{closure::Closure, JsCast, JsValue},
    worker_sys::web_sys::{BinaryType, MessageEvent, WebSocket},
};

// In the connect function, update the onmessage handler:
let onmessage_tx = read_tx.clone();
let onmessage = Closure::wrap(Box::new(move |evt: MessageEvent| {
    let data = evt.data();
    
    // Handle ArrayBuffer
    if let Ok(arr) = data.dyn_into::<ArrayBuffer>() {
        let _ = onmessage_tx.send(WebSocketMessage::Message(Uint8Array::new(&arr).to_vec()));
    } 
    // Handle Blob (fallback)
    else if let Ok(blob) = data.dyn_into::<worker::js_sys::Blob>() {
        // Convert blob to ArrayBuffer asynchronously
        let tx = onmessage_tx.clone();
        spawn_local(async move {
            match worker::js_sys::JsFuture::from(blob.array_buffer()).await {
                Ok(arr_buffer) => {
                    if let Ok(arr) = arr_buffer.dyn_into::<ArrayBuffer>() {
                        let _ = tx.send(WebSocketMessage::Message(Uint8Array::new(&arr).to_vec()));
                    }
                }
                Err(e) => console_warn!("Failed to convert blob: {:?}", e),
            }
        });
    }
}) as Box<dyn Fn(MessageEvent)>);
