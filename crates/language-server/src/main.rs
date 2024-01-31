mod db;
mod diagnostics;
mod goto;
mod server;
mod state;
mod util;
mod workspace;

use db::Jar;
mod handlers {
    pub mod notifications;
    pub mod request;
}

use server::run_server;

fn main() {
    let _ = run_server();
}