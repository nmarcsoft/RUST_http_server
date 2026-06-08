mod server;

fn main() {
    match server::init_tcp_listener() {
        Ok(_) => println!("Server stopped"),
        Err(e) => println!("Error: {}", e),
    }
}
