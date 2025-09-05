use clipboard_rs::{
    Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
    WatcherShutdown,
};
use mouse_position::mouse_position::Mouse;
use std::sync::{Mutex, mpsc};

pub(crate) struct Event {
    pub text: String,
    pub mouse_x: f32,
    pub mouse_y: f32,
}

pub(crate) struct Observer {
    ctx: ClipboardContext,
    event_tx: mpsc::Sender<Event>,
}

impl Observer {
    pub fn new(event_tx: mpsc::Sender<Event>) -> Self {
        let ctx = ClipboardContext::new().unwrap();
        Observer { ctx, event_tx }
    }
}

impl ClipboardHandler for Observer {
    fn on_clipboard_change(&mut self) {
        match self.ctx.get_text() {
            Ok(txt) => {
                // Get mouse position when clipboard changes
                let mouse_position = Mouse::get_mouse_position();
                let (x, y) = match mouse_position {
                    Mouse::Position { x, y } => (x as f32, y as f32),
                    _ => (0.0, 0.0),
                };

                // Send text and position through channel
                let _ = self.event_tx.send(Event {
                    text: txt,
                    mouse_x: x,
                    mouse_y: y,
                });
            }
            Err(e) => eprintln!("on_clipboard_change, error = {}", e),
        }
    }
}

pub(crate) fn set_clipboard_text(text: String) -> anyhow::Result<()> {
    if let Ok(ctx) = ClipboardContext::new() {
        if let Err(e) = ctx.set_text(text) {
            return Err(anyhow::anyhow!("failed to set clipboard text: {}", e));
        }
    } else {
        return Err(anyhow::anyhow!("Failed to get clipboard context"));
    }
    Ok(())
}

pub(crate) fn start_observer(event_tx: mpsc::Sender<Event>) -> Mutex<Option<WatcherShutdown>> {
    // Start clipboard watcher in a separate thread
    let manager = Observer::new(event_tx);
    let mut watcher = ClipboardWatcherContext::new().unwrap();
    let shutdown = watcher.add_handler(manager).get_shutdown_channel();
    let shutdown = Mutex::new(Some(shutdown));

    std::thread::spawn(move || {
        println!("clipboard observer started...");
        watcher.start_watch();
    });

    shutdown
}
