#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq)]
pub enum Message {
    None,
    Quit,
    HelloWorld(String),
}
