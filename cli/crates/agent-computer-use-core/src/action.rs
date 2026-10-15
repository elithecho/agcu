use crate::node::Point;
use crate::selector::Selector;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Click {
        #[serde(skip_serializing_if = "Option::is_none")]
        selector: Option<Selector>,

        #[serde(skip_serializing_if = "Option::is_none")]
        coordinates: Option<Point>,

        #[serde(default)]
        button: MouseButton,

        #[serde(default = "default_click_count")]
        count: u32,
    },

    Type {
        text: String,
        /// Application target for keyboard fallback when no element is selected.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        app: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        selector: Option<Selector>,

        #[serde(default)]
        submit: bool,
    },

    KeyPress {
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        app: Option<String>,
    },

    Scroll {
        direction: ScrollDirection,
        #[serde(default = "default_scroll_amount")]
        amount: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        selector: Option<Selector>,
        #[serde(skip_serializing_if = "Option::is_none")]
        app: Option<String>,
    },

    MoveMouse {
        #[serde(skip_serializing_if = "Option::is_none")]
        selector: Option<Selector>,
        #[serde(skip_serializing_if = "Option::is_none")]
        coordinates: Option<Point>,
    },

    Drag {
        from: Point,
        to: Point,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        app: Option<String>,
    },

    Focus {
        selector: Selector,
    },
    Screenshot {
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        app: Option<String>,
        #[serde(default, skip_serializing_if = "is_false")]
        grid: bool,
    },
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    #[default]
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ScrollDirection {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

fn default_click_count() -> u32 {
    1
}

#[cfg(test)]
mod pid_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn optional_targets_preserve_old_action_json() {
        for legacy in [
            json!({"type":"type", "text":"hello", "submit":false}),
            json!({"type":"drag", "from":{"x":1.0,"y":2.0}, "to":{"x":3.0,"y":4.0}}),
        ] {
            let action: Action = serde_json::from_value(legacy.clone()).unwrap();
            assert_eq!(serde_json::to_value(action).unwrap(), legacy);
        }
        let targeted = json!({"type":"type", "text":"hello", "app":"pid:42", "submit":true});
        let action: Action = serde_json::from_value(targeted.clone()).unwrap();
        assert_eq!(serde_json::to_value(action).unwrap(), targeted);
    }
}

fn default_scroll_amount() -> u32 {
    3
}

fn is_false(value: &bool) -> bool {
    !value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screenshot_old_payload_and_grid_round_trip() {
        let old = r#"{"type":"screenshot","path":"plain.png","app":"App"}"#;
        let action: Action = serde_json::from_str(old).unwrap();
        assert!(matches!(action, Action::Screenshot { grid: false, .. }));
        assert_eq!(serde_json::to_string(&action).unwrap(), old);
        let grid = r#"{"type":"screenshot","grid":true}"#;
        let action: Action = serde_json::from_str(grid).unwrap();
        assert!(matches!(action, Action::Screenshot { grid: true, .. }));
        assert_eq!(serde_json::to_string(&action).unwrap(), grid);
    }
}
