pub fn greet(name: Option<&str>) -> String {
    if let Some(name) = name {
        format!("Hello, {name}!")
    } else{
        "Hello, world".to_string()
    }
}