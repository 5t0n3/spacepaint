mod state;

const SHADER_TICKS_PER_STATE_TICK: u32 = 20;

#[tokio::main]
async fn main() {
    let mut state = state::State::load_from_image("images/scribble-map4.png")
        .await
        .expect("couldn't create state");

    println!("post state load");
    state
        .tick_state_by_count(SHADER_TICKS_PER_STATE_TICK * 600 * 2)
        .await
        .expect("couldn't tick state");

    println!("saving");
    state
        .save_state_to_image("images/scribble-4.5.png")
        .await
        .expect("couldn't save image");
}
