use egui::{Event as EguiEvent, Key as EguiKey, Modifiers, MouseWheelUnit, Pos2, Rect};
use maple_engine::{
    prelude::{Input, MouseButton, TouchPhase},
    resources::KeyCode,
};

/// Builds an `egui::RawInput` for this frame purely from Input's public API.
/// Call this *before* `input.end_frame()` clears the per-frame state.
pub fn input_to_egui_raw_input(
    input: &Input,
    time: f64,
    feed_pointer_events: bool,
) -> egui::RawInput {
    let modifiers = egui_modifiers(input);
    let mut events = Vec::new();

    // --- Keyboard ---
    for &code in &input.key_just_pressed {
        if let Some(key) = winit_key_to_egui(code) {
            events.push(EguiEvent::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            });
        }
    }
    for &code in &input.key_just_released {
        if let Some(key) = winit_key_to_egui(code) {
            events.push(EguiEvent::Key {
                key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers,
            });
        }
    }

    // --- Text input ---
    if !input.text_input.is_empty() {
        events.push(EguiEvent::Text(input.text_input.clone()));
    }

    // --- Pointer ---
    if feed_pointer_events {
        let pos = Pos2::new(
            input.cursor_position_points().x,
            input.cursor_position_points().y,
        );

        if input.cursor_exit {
            events.push(EguiEvent::PointerGone);
        } else {
            events.push(EguiEvent::PointerMoved(pos));
        }

        for &button in &input.mouse_button_just_pressed {
            if let Some(egui_button) = winit_mouse_button_to_egui(button) {
                events.push(EguiEvent::PointerButton {
                    pos,
                    button: egui_button,
                    pressed: true,
                    modifiers,
                });
            }
        }
        for &button in &input.mouse_button_just_released {
            if let Some(egui_button) = winit_mouse_button_to_egui(button) {
                events.push(EguiEvent::PointerButton {
                    pos,
                    button: egui_button,
                    pressed: false,
                    modifiers,
                });
            }
        }

        // --- Scroll ---
        if let Some(phase) = input.scroll_phase {
            let egui_phase = winit_touch_phase_to_egui(phase);

            if input.scroll_delta_lines != glam::vec2(0.0, 0.0) {
                events.push(EguiEvent::MouseWheel {
                    unit: MouseWheelUnit::Line,
                    delta: egui::vec2(input.scroll_delta_lines.x, input.scroll_delta_lines.y),
                    phase: egui_phase,
                    modifiers,
                });
            }
            if input.scroll_delta_pixels != glam::vec2(0.0, 0.0) {
                events.push(EguiEvent::MouseWheel {
                    unit: MouseWheelUnit::Point,
                    delta: egui::vec2(input.scroll_delta_pixels.x, input.scroll_delta_pixels.y),
                    phase: egui_phase,
                    modifiers,
                });
            }
        }
    }

    let size_points = input.screen_size_points();
    let screen_rect = Rect::from_min_size(Pos2::ZERO, egui::vec2(size_points.x, size_points.y));

    egui::RawInput {
        screen_rect: Some(screen_rect),
        time: Some(time),
        events,
        modifiers,
        ..Default::default()
    }
}

fn egui_modifiers(input: &Input) -> Modifiers {
    let ctrl =
        input.keys.contains(&KeyCode::ControlLeft) || input.keys.contains(&KeyCode::ControlRight);
    Modifiers {
        alt: input.keys.contains(&KeyCode::AltLeft) || input.keys.contains(&KeyCode::AltRight),
        ctrl,
        shift: input.keys.contains(&KeyCode::ShiftLeft)
            || input.keys.contains(&KeyCode::ShiftRight),
        mac_cmd: false,
        command: ctrl,
    }
}

fn winit_touch_phase_to_egui(phase: TouchPhase) -> egui::TouchPhase {
    match phase {
        TouchPhase::Started => egui::TouchPhase::Start,
        TouchPhase::Moved => egui::TouchPhase::Move,
        TouchPhase::Ended => egui::TouchPhase::End,
        TouchPhase::Cancelled => egui::TouchPhase::Cancel,
    }
}

fn winit_mouse_button_to_egui(button: MouseButton) -> Option<egui::PointerButton> {
    match button {
        MouseButton::Left => Some(egui::PointerButton::Primary),
        MouseButton::Right => Some(egui::PointerButton::Secondary),
        MouseButton::Middle => Some(egui::PointerButton::Middle),
        MouseButton::Back => Some(egui::PointerButton::Extra1),
        MouseButton::Forward => Some(egui::PointerButton::Extra2),
        _ => None,
    }
}

fn winit_key_to_egui(code: KeyCode) -> Option<EguiKey> {
    Some(match code {
        KeyCode::ArrowDown => EguiKey::ArrowDown,
        KeyCode::ArrowLeft => EguiKey::ArrowLeft,
        KeyCode::ArrowRight => EguiKey::ArrowRight,
        KeyCode::ArrowUp => EguiKey::ArrowUp,
        KeyCode::Escape => EguiKey::Escape,
        KeyCode::Tab => EguiKey::Tab,
        KeyCode::Backspace => EguiKey::Backspace,
        KeyCode::Enter => EguiKey::Enter,
        KeyCode::Space => EguiKey::Space,
        KeyCode::Delete => EguiKey::Delete,
        KeyCode::Insert => EguiKey::Insert,
        KeyCode::Home => EguiKey::Home,
        KeyCode::End => EguiKey::End,
        KeyCode::PageUp => EguiKey::PageUp,
        KeyCode::PageDown => EguiKey::PageDown,
        KeyCode::Minus => EguiKey::Minus,
        KeyCode::Equal => EguiKey::Equals,
        KeyCode::F1 => EguiKey::F1,
        KeyCode::F2 => EguiKey::F2,
        KeyCode::F3 => EguiKey::F3,
        KeyCode::F4 => EguiKey::F4,
        KeyCode::F5 => EguiKey::F5,
        KeyCode::F6 => EguiKey::F6,
        KeyCode::F7 => EguiKey::F7,
        KeyCode::F8 => EguiKey::F8,
        KeyCode::F9 => EguiKey::F9,
        KeyCode::F10 => EguiKey::F10,
        KeyCode::F11 => EguiKey::F11,
        KeyCode::F12 => EguiKey::F12,
        KeyCode::Digit0 => EguiKey::Num0,
        KeyCode::Digit1 => EguiKey::Num1,
        KeyCode::Digit2 => EguiKey::Num2,
        KeyCode::Digit3 => EguiKey::Num3,
        KeyCode::Digit4 => EguiKey::Num4,
        KeyCode::Digit5 => EguiKey::Num5,
        KeyCode::Digit6 => EguiKey::Num6,
        KeyCode::Digit7 => EguiKey::Num7,
        KeyCode::Digit8 => EguiKey::Num8,
        KeyCode::Digit9 => EguiKey::Num9,
        KeyCode::KeyA => EguiKey::A,
        KeyCode::KeyB => EguiKey::B,
        KeyCode::KeyC => EguiKey::C,
        KeyCode::KeyD => EguiKey::D,
        KeyCode::KeyE => EguiKey::E,
        KeyCode::KeyF => EguiKey::F,
        KeyCode::KeyG => EguiKey::G,
        KeyCode::KeyH => EguiKey::H,
        KeyCode::KeyI => EguiKey::I,
        KeyCode::KeyJ => EguiKey::J,
        KeyCode::KeyK => EguiKey::K,
        KeyCode::KeyL => EguiKey::L,
        KeyCode::KeyM => EguiKey::M,
        KeyCode::KeyN => EguiKey::N,
        KeyCode::KeyO => EguiKey::O,
        KeyCode::KeyP => EguiKey::P,
        KeyCode::KeyQ => EguiKey::Q,
        KeyCode::KeyR => EguiKey::R,
        KeyCode::KeyS => EguiKey::S,
        KeyCode::KeyT => EguiKey::T,
        KeyCode::KeyU => EguiKey::U,
        KeyCode::KeyV => EguiKey::V,
        KeyCode::KeyW => EguiKey::W,
        KeyCode::KeyX => EguiKey::X,
        KeyCode::KeyY => EguiKey::Y,
        KeyCode::KeyZ => EguiKey::Z,
        _ => return None,
    })
}
