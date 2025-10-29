use crate::editor::NOTIFICATION_SENDER;

pub fn send_notification(notification: impl ToString) {
    if let Ok(notification_producer) = NOTIFICATION_SENDER.lock() {
        if let Some(sender) = &*notification_producer {
            let _ = sender.send(notification.to_string());
        }
    }
}
