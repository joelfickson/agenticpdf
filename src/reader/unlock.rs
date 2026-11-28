use super::{Reader, chrome::Input, tabs::Lock};
use gpui::{Context, Window};

impl Reader {
    pub(super) fn submit_password(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.chrome.password.text.is_empty() {
            return;
        }
        let Some(tab) = self
            .tabs
            .current_mut()
            .filter(|tab| matches!(tab.lock, Some(Lock::Asking | Lock::Wrong)))
        else {
            return;
        };
        tab.lock = Some(Lock::Checking);
        self.worker.unlock(
            tab.session,
            tab.path.clone(),
            self.chrome.password.text.clone(),
        );
        self.reset_request(window);
        self.request(window);
        cx.notify();
    }

    pub(super) fn cancel_password(&mut self, cx: &mut Context<Self>) {
        if let Some(tab) = self
            .tabs
            .current_mut()
            .filter(|tab| matches!(tab.lock, Some(Lock::Asking | Lock::Wrong)))
        {
            tab.lock = Some(Lock::Cancelled);
        }
        self.forget_typed_password();
        cx.notify();
    }

    pub(super) fn ask_password(&mut self, cx: &mut Context<Self>) {
        if let Some(tab) = self
            .tabs
            .current_mut()
            .filter(|tab| tab.lock == Some(Lock::Cancelled))
        {
            tab.lock = Some(Lock::Asking);
            self.chrome.password.clear();
            self.chrome.input = Input::Password;
        }
        cx.notify();
    }

    pub(super) fn toggle_password_shown(&mut self, cx: &mut Context<Self>) {
        self.chrome.password_shown = !self.chrome.password_shown;
        cx.notify();
    }

    pub(super) fn focus_password(&mut self, cx: &mut Context<Self>) {
        if self
            .tabs
            .current()
            .is_some_and(|tab| matches!(tab.lock, Some(Lock::Asking | Lock::Wrong)))
        {
            self.chrome.input = Input::Password;
            cx.notify();
        }
    }

    pub(super) fn sync_password_input(&mut self) {
        self.chrome.password.clear();
        self.chrome.password_shown = false;
        let asking = self
            .tabs
            .current()
            .is_some_and(|tab| matches!(tab.lock, Some(Lock::Asking | Lock::Wrong)));
        if asking {
            self.chrome.input = Input::Password;
        } else if self.chrome.input == Input::Password {
            self.chrome.input = Input::None;
        }
    }

    fn forget_typed_password(&mut self) {
        self.chrome.password.clear();
        self.chrome.password_shown = false;
        if self.chrome.input == Input::Password {
            self.chrome.input = Input::None;
        }
    }
}
