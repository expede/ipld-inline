cargo_component_bindings::generate!();

use bindings::Guest;

struct Component;

impl Guest for Component {
    /// Say hello!
    fn hello_world() -> String {
        "Hello, World!!".to_string()
    }
}

// impl Component {
//     async fn run(self) -> anyhow::Result<()> {
//         let hey = Self::hello_world();
//         println!("{hey}");
//         Ok(())
//     }
// }
//
// #[async_std::main]
// async fn main() -> anyhow::Result<()> {
//     Component.run().await
// }
