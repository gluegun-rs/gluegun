use exports::squared::hello_world::greet::Guest;

wit_bindgen::generate!({
    world: "host"
});

struct Host;

impl Guest for Host {
    fn greet(name: Option<String>) -> String {
        hello_world::greet(name.as_deref())
    }
}

export!(Host);
