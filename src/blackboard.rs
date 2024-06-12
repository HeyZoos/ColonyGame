#[derive(bevy::prelude::Component, Debug)]
pub struct Blackboard(serde_json::Value);

impl Blackboard {
    /// Creates a new, empty `Blackboard`.
    ///
    /// # Examples
    ///
    /// ```
    /// bevy_game::blackboard::Blackboard::new();
    /// ```
    pub fn new() -> Self {
        Self(serde_json::json!({}))
    }

    /// Gets a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut blackboard = bevy_game::blackboard::Blackboard::new();
    /// blackboard.set("key", serde_json::json!("value"));
    /// assert_eq!(blackboard.get("key"), &serde_json::json!("value"));
    /// ```
    pub fn get<I: serde_json::value::Index>(&self, key: I) -> &serde_json::Value {
        self.0.get(key).unwrap()
    }

    /// Sets a key-value pair in the `Blackboard`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut blackboard = bevy_game::blackboard::Blackboard::new();
    /// blackboard.set("key", serde_json::json!("value"));
    /// assert_eq!(blackboard.get("key"), &serde_json::json!("value"));
    /// ```
    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        self.0.as_object_mut().unwrap().insert(key.to_string(), value);
    }
}
