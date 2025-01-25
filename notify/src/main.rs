use notify_rust::Notification;

fn main() {
    Notification::new()
        .summary("Hello from Rust!")
        .body("This is a test notification.")
        .show()
        .unwrap();
}
