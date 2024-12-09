use std::{cell::OnceCell, io::Cursor, sync::{Mutex, OnceLock}};

use flexbuffers::{FlexbufferSerializer, Reader};
use image::{ColorType, ImageReader};

use rand::{thread_rng, RngCore};
use wasm_bindgen::prelude::*;
use web_sys::{js_sys, ErrorEvent, MessageEvent, WebSocket};
use serde::{Serialize, Deserialize};

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Pixel {
    temp: u8,
    haze: u8,
    wind: (u8, u8)
}

#[wasm_bindgen]
extern {
    fn alert(s: &str);

    fn update_map(data: Vec<Pixel>, width: u32, area: Rect);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
}

static CLIENT_ID: OnceLock<u64> = OnceLock::new();

#[wasm_bindgen]
pub fn do_changes(points: Vec<Pixel>, width_degrees: f64) {
    todo!()
}

#[wasm_bindgen]
pub fn update_viewport(rect: Rect) {
    send_packet(Packet::Viewport { area: rect, client_id: *CLIENT_ID.get().unwrap() })
}

fn send_packet(p: Packet) {
    let mut s = FlexbufferSerializer::new();
    p.serialize(&mut s).unwrap();
    SOCK.lock().unwrap().clone().unwrap().sock.send_with_u8_array(s.view()).unwrap();
}


#[derive(Serialize, Deserialize)]
pub enum ModificationType {
    Heat, Cool, Humidify, Dehumidify, Wind
}

#[derive(Serialize, Deserialize)]
pub struct PNGFile(pub Vec<u8>);

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct LatLong {
    pub lat: f64,
    pub long: f64
}

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub top_left: LatLong,
    pub bottom_right: LatLong
}

#[derive(Serialize, Deserialize)]
pub enum Packet {
    Snapshot { data: PNGFile, location: Rect },
    Modification { tpe: ModificationType, points: Vec<LatLong>, brush_size_degrees: f64, location: Rect, client_id: u64 },
    Viewport { area: Rect, client_id: u64 }
}

fn handle_packet(pack: Vec<u8>) -> Option<()> {
    let r = Reader::get_root(pack.as_slice()).ok()?;
    let p = Packet::deserialize(r).ok()?;

    match p {
        Packet::Snapshot { data, location } => {
            let img = ImageReader::new(Cursor::new(data.0)).decode().ok()?;
            if img.color() != ColorType::Rgba8 || img.width() * img.height() > 8192 {
                return None;
            }

            let im = img.as_rgba8().unwrap();

            let out = im.pixels().map(|x| Pixel { temp: x.0[0], haze: x.0[3], wind: (x.0[1], x.0[2]) }).collect();

            update_map(out, im.width(), location);
        }
        // other packet types are ignored by the client
        _ => {}
    }

    Some(())
}

#[derive(Clone)]
struct WS {
    sock: WebSocket
}

unsafe impl Send for WS {}

static SOCK: Mutex<Option<WS>> = Mutex::new(None);

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    CLIENT_ID.set(rand::random()).unwrap();

    let ws = WebSocket::new("wss://echo.websocket.org")?;
    ws.set_binary_type(web_sys::BinaryType::Blob);

    *SOCK.lock().unwrap() = Some(WS { sock: ws.clone() });

    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            console_log!("message event, received arraybuffer: {:?}", abuf);
            let array = js_sys::Uint8Array::new(&abuf).to_vec();
            handle_packet(array);
        } else if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
            console_log!("message event, received blob: {:?}", blob);
            // better alternative to juggling with FileReader is to use https://crates.io/crates/gloo-file
            let fr = web_sys::FileReader::new().unwrap();
            let fr_c = fr.clone();
            // create onLoadEnd callback
            let onloadend_cb = Closure::<dyn FnMut(_)>::new(move |_e: web_sys::ProgressEvent| {
                let array = js_sys::Uint8Array::new(&fr_c.result().unwrap()).to_vec();
                handle_packet(array);
            });
            fr.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
            fr.read_as_array_buffer(&blob).expect("blob not readable");
            onloadend_cb.forget();
        } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            console_log!("message event, received Text: {:?}", txt);
        } else {
            console_log!("message event, received Unknown: {:?}", e.data());
        }
    });

    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        console_log!("error event: {:?}", e);
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let cloned_ws = ws.clone();
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        console_log!("socket opened");
        match cloned_ws.send_with_str("ping") {
            Ok(_) => console_log!("message successfully sent"),
            Err(err) => console_log!("error sending message: {:?}", err),
        }
        // send off binary message
        match cloned_ws.send_with_u8_array(&[0, 1, 2, 3]) {
            Ok(_) => console_log!("binary message successfully sent"),
            Err(err) => console_log!("error sending message: {:?}", err),
        }
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    Ok(())
}

