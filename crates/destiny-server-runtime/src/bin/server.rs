use std::sync::Arc;

use destiny_runtime_core::Runtime;
use destiny_server_runtime::ServerRuntime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ProjectDestiny server runtime starting...");

    let runtime = Runtime::open("destiny_runtime.db")?;

    let server = ServerRuntime::new(Arc::new(runtime));

    server.start();

    println!("ProjectDestiny server runtime active.");

    loop {
        std::thread::park();
    }
}
