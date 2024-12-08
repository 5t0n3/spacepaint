mod state;

#[tokio::main]
async fn main() {
    let mut state = state::State::load_from_image("scribble-map2.png")
        .await
        .expect("couldn't create state");

    println!("post state load");
    state.tick_state().await.expect("couldn't tick state");
    println!("post tick");

    state
        .save_state_to_image("scribble-2.png")
        .await
        .expect("couldn't save image");
}
