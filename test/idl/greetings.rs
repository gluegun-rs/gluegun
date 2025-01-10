pub struct Greetings {
    language: String,
}

impl Greetings {
    pub fn new() -> Self {
        Self {
            language: "en".to_string(),
        }
    }

    pub fn language(mut self, language: &str) -> Self {
        self.language = language.to_string();
        self
    }

    pub fn greet(self, name: String) -> anyhow::Result<String>  {
        match &self.language {
            "en" => Ok(format!("Hello, {name}!")),
            "es" => Ok(format!("Hola, {name}!")),
            _ => anyhow::bail!("unknown language {language}"),
        }
    }
}