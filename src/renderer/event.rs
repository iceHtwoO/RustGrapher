use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use winit::event::{MouseButton, VirtualKeyCode};

pub struct InputEvent {
    initial: bool,
    start_time: Instant,
}

impl InputEvent {
    pub fn new() -> Self {
        Self {
            initial: true,
            start_time: Instant::now(),
        }
    }

    pub fn time_engaged(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn reset_timer(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn is_initial_check(&mut self) -> bool {
        if self.initial {
            self.initial = false;
            return true;
        }
        false
    }
}

pub struct EventManager {
    key_event: HashMap<VirtualKeyCode, InputEvent>,
    mouse_event: HashMap<MouseButton, InputEvent>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            key_event: HashMap::new(),
            mouse_event: HashMap::new(),
        }
    }

    pub fn insert_key(&mut self, vk: VirtualKeyCode) {
        self.key_event.insert(vk, InputEvent::new());
    }

    pub fn remove_key(&mut self, vk: &VirtualKeyCode) {
        self.key_event.remove(vk);
    }

    #[allow(dead_code)]
    pub fn contains_key(&self, vk: &VirtualKeyCode) -> bool {
        self.key_event.contains_key(vk)
    }

    #[allow(dead_code)]
    pub fn get_key_event(&mut self, vk: &VirtualKeyCode) -> Option<&InputEvent> {
        self.key_event.get(vk)
    }

    pub fn get_key_event_mut(&mut self, vk: &VirtualKeyCode) -> Option<&mut InputEvent> {
        self.key_event.get_mut(vk)
    }

    pub fn insert_mouse_button(&mut self, mb: MouseButton) {
        self.mouse_event.insert(mb, InputEvent::new());
    }

    pub fn remove_mouse_button(&mut self, mb: &MouseButton) {
        self.mouse_event.remove(mb);
    }

    #[allow(dead_code)]
    pub fn contains_mouse_button(&self, mb: &MouseButton) -> bool {
        self.mouse_event.contains_key(mb)
    }

    #[allow(dead_code)]
    pub fn get_mouse_button_event(&self, mb: &MouseButton) -> Option<&InputEvent> {
        self.mouse_event.get(mb)
    }

    pub fn get_mouse_button_event_mut(&mut self, mb: &MouseButton) -> Option<&mut InputEvent> {
        self.mouse_event.get_mut(mb)
    }
}
