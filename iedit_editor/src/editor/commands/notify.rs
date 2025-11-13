use crate::editor::NOTIFICATION_SENDER;
use crate::input::Notification;

pub fn send_notification(notification: Notification) {
    if let Ok(notification_producer) = NOTIFICATION_SENDER.lock()
        && let Some(sender) = &*notification_producer
    {
        let _ = sender.send(notification);
    }
}

#[inline]
pub fn send_simple_notification(message: impl ToString) {
    send_notification(Notification::Simple(message.to_string()));
}
