use crate::editor::NOTIFICATION_SENDER;
use crate::input::Notification;

pub fn send_simple_notification(message: impl ToString) {
    if let Ok(notification_producer) = NOTIFICATION_SENDER.lock()
        && let Some(sender) = &*notification_producer
    {
        let _ = sender.send(Notification::Simple(message.to_string()));
    }
}
