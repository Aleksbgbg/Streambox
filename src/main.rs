mod server;
mod types;
mod utils;

use crate::server::http_server;
use crate::types::result::Result;

fn main() -> Result<()> {
  http_server::run_http_server()
}
