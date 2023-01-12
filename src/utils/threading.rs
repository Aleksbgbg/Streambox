use std::thread;
use std::thread::JoinHandle;

pub fn spawn_thread<F, T>(handler: F) -> JoinHandle<()>
where
  F: FnOnce() -> T,
  F: Send + 'static,
{
  thread::spawn(|| {
    handler();
  })
}
