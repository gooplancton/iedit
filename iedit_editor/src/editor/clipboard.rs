use copypasta_ext::copypasta::ClipboardContext;
use copypasta_ext::copypasta::ClipboardProvider;
use copypasta_ext::wayland_bin::WaylandBinClipboardContext;
use copypasta_ext::x11_bin::X11BinClipboardContext;
use iedit_document::Text;

#[derive(Default)]
pub struct BuiltinClipboardContext {
    pub text: Option<Text>,
}

pub trait EditorClipboard {
    fn get_text(&mut self) -> Option<Text>;
    fn set_text(&mut self, text: Text);
}

impl EditorClipboard for BuiltinClipboardContext {
    fn get_text(&mut self) -> Option<Text> {
        self.text.clone()
    }

    fn set_text(&mut self, text: Text) {
        self.text = Some(text)
    }
}

impl<T: ClipboardProvider> EditorClipboard for T {
    fn get_text(&mut self) -> Option<Text> {
        self.get_contents().map(Text::from).ok()
    }

    fn set_text(&mut self, text: Text) {
        let _ = self.set_contents(text.into());
    }
}

pub fn get_clipboard(use_system_clipboard: bool) -> Box<dyn EditorClipboard> {
    if !use_system_clipboard {
        return Box::from(BuiltinClipboardContext::default());
    }

    if let Ok(ctx) = ClipboardContext::new() {
        Box::from(ctx)
    } else if let Ok(ctx) = WaylandBinClipboardContext::new() {
        Box::from(ctx)
    } else if let Ok(ctx) = X11BinClipboardContext::new() {
        Box::from(ctx)
    } else {
        Box::from(BuiltinClipboardContext::default())
    }
}
