use flexbuffers::Reader;
use futures::{SinkExt, StreamExt};
use log::debug;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use warp::ws::{self, WebSocket};
use warp::Filter;

mod message;
mod state;

struct GlobalState {
    map: state::State,
    viewport: Option<message::Rect>,
    client: Option<futures::stream::SplitSink<WebSocket, ws::Message>>,
}

fn start_syncing(
    websocket: ws::Ws,
    state_shard: Arc<Mutex<GlobalState>>,
    modification_sink: tokio::sync::mpsc::Sender<message::Packet>,
) -> impl warp::Reply {
    websocket.on_upgrade(move |actual_ws: WebSocket| async move {
        // split websocket into stream and sink ends
        let (sink, mut stream) = actual_ws.split();

        // add sink to global state to send updates to
        {
            let mut locked_state = state_shard.lock().await;
            locked_state.client = Some(sink);
        }

        // task to process incoming messages
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                let message = message.expect("couldn't receive message from websocket");

                let packet = message.as_bytes();

                if message.is_binary() {
                    let message_reader = Reader::get_root(packet)
                        .expect("couldn't construct flexbuffer reader for packet body");
                    let payload = message::Packet::deserialize(message_reader)
                        .expect("couldn't deserialize packet from websocket message");

                    match payload {
                        message::Packet::Snapshot { .. } => {
                            panic!("client shouldn't send snapshots")
                        }
                        modif @ message::Packet::Modification { .. } => modification_sink
                            .send(modif)
                            .await
                            .expect("couldn't send modification to sink"),
                        message::Packet::Viewport { area, .. } => {
                            let mut locked_state = state_shard.lock().await;
                            locked_state.viewport = Some(area);
                        }
                    }
                } else {
                    debug!("non binary message: {packet:?}");
                }
            }
        });
    })
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let (mod_sender, mut mod_queue) = tokio::sync::mpsc::channel(50);
    let global_state = GlobalState {
        map: state::State::load_from_image("images/just-noise.png")
            .await
            .expect("couldn't load state image"),
        viewport: None,
        client: None,
    };
    let global_state = Arc::new(Mutex::new(global_state));

    // yay lifetimes and ownershpi
    let global_state_modification = global_state.clone();
    let global_state_ticking = global_state.clone();
    let global_state_saving = global_state.clone();
    let global_state_clone_wsroute = global_state.clone();

    // spawn task to apply modifications
    tokio::spawn(async move {
        while let Some(_modif @ message::Packet::Modification { .. }) = mod_queue.recv().await {
            // TODO: apply modification to state
            // global_state_clone.method();
            let _ = global_state_modification;
        }
    });

    // also spawn task to step internal state every half second
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(500));

        loop {
            interval.tick().await;

            {
                let mut locked_state = global_state_ticking.lock().await;

                locked_state
                    .map
                    .tick_state_by_count(1)
                    .await
                    .expect("couldn't tick state");

                if let Some(rect) = locked_state.viewport {
                    let snapshot_png = locked_state
                        .map
                        .render_cropped_state(rect)
                        .expect("couldn't render cropped state");
                    println!("png length = {} bytes", snapshot_png.len());

                    let packet = message::Packet::Snapshot {
                        data: message::PNGFile(snapshot_png),
                        location: rect,
                    };
                    let mut serializer = flexbuffers::FlexbufferSerializer::new();
                    packet
                        .serialize(&mut serializer)
                        .expect("couldn't serialize snapshot packet");

                    if let Some(client_ws) = locked_state.client.as_mut() {
                        client_ws
                            .send(ws::Message::binary(serializer.view()))
                            .await
                            .expect("couldn't send message to client websocket");
                    }
                }
            }
        }
    });

    // *also* spawn task to save state to file every 10 seconds
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            let state_data = {
                let locked_state = global_state_saving.lock().await;
                locked_state.map.get_state_clone()
            };

            state::State::save_raw_to_image(state_data, "state.png")
                .expect("couldn't save state image");
        }
    });

    // TODO: read from frontend/just serve actual files
    let index_route = warp::path::end().and(warp::fs::file("../frontend/index.html"));
    let about_route = warp::path("about")
        .and(warp::path::end())
        .and(warp::fs::file("../frontend/about.html"));
    let static_route = warp::fs::dir("../frontend/");
    let ws_route = warp::path("sync")
        .and(warp::path::end())
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let state_clone = global_state_clone_wsroute.clone();
            start_syncing(ws, state_clone, mod_sender.clone())
        });

    let all_filters = index_route.or(about_route).or(static_route).or(ws_route);

    let bind_address: SocketAddr = std::env::var("SPACEPAINT_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:5000".to_owned())
        .parse()
        .expect("invalid socket addr");
    warp::serve(all_filters).run(bind_address).await
}
