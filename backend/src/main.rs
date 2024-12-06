use futures::SinkExt;
use log::{debug, error};
use std::net::SocketAddr;
use std::time::Duration;
use warp::ws::{self, WebSocket};
use warp::Filter;

fn start_syncing(websocket: ws::Ws, address: Option<SocketAddr>) -> impl warp::Reply {
    debug!("incoming websocket connection from {address:?}");

    websocket.on_upgrade(|mut actual_ws: WebSocket| async {
        tokio::spawn(async {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                interval.tick().await;
                if let Err(e) = actual_ws
                    .send(ws::Message::text("consider yourself synced"))
                    .await
                {
                    error!("websocket sending failed: {e:?}");
                    break;
                }
            }

            actual_ws.close().await.expect("error closing ws");
        });
    })
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // TODO: warp's log filter
    let index_route = warp::path::end().map(|| "index");
    let about_route = warp::path("about").and(warp::path::end()).map(|| "about");
    let ws_route = warp::path("sync")
        .and(warp::ws())
        .and(warp::addr::remote())
        .map(start_syncing);

    // TODO:

    // TODO: schedule repeating task as proof of concept? probably tokio interval + spawn_task
    // (to represent saving to file)

    let all_filters = index_route.or(about_route).or(ws_route);

    let bind_address_str: SocketAddr = std::env::var("SPACEPAINT_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:5000".to_owned())
        .parse()
        .expect("invalid socket addr");
    warp::serve(all_filters).run(bind_address_str).await
}
