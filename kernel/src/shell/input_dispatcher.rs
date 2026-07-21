use pc_keyboard::DecodedKey;

pub struct InputDispatcher {
    focus: InputTarget,
}

impl InputDispatcher {
    pub fn new() -> Self {
        InputDispatcher {
            focus: InputTarget::Shell,
        }
    }

    pub fn dispatch_key(&mut self, key: DecodedKey) {
        match self.focus {
            InputTarget::Shell => {
                super::SHELL.lock().handle_input(key);
            }
            InputTarget::_Process(_) => {}
            InputTarget::_None => {
                // No target set, ignore the key input
            }
        }
    }

    fn _set_focus(&mut self, target: InputTarget) {
        self.focus = target;
    }
}

enum InputTarget {
    Shell,
    _Process(u32),
    _None,
}

lazy_static::lazy_static! {
    pub static ref INPUT_DISPATCHER: spin::Mutex<InputDispatcher> = spin::Mutex::new(InputDispatcher::new());
}
