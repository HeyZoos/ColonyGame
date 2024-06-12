/// A `Blackboard` is a storage component designed for AI agents to read and store arbitrary data.
/// It utilizes a JSON-like structure to allow flexible and dynamic data management.
///
/// The `Blackboard` struct wraps a `serde_json::Value` which can hold any valid JSON data.
/// This makes it suitable for storing a wide variety of data types and structures that an AI agent
/// might need to use or modify during its operation.
///
/// # Examples
///
/// ```
/// use bevy_game::blackboard::Blackboard;
///
/// let mut blackboard = Blackboard::new();
/// blackboard.set("health", serde_json::json!(100));
/// blackboard.set("name", serde_json::json!("Agent Smith"));
///
/// let health = blackboard.get("health");
/// println!("Agent health: {}", health);
///
/// let name = blackboard.get("name");
/// println!("Agent name: {}", name);
/// ```
#[derive(bevy::prelude::Component, Debug)]
pub struct Blackboard(serde_json::Value);

impl Blackboard {
    /// Creates a new, empty `Blackboard`.
    ///
    /// This initializes the `Blackboard` with an empty JSON object, ready to store key-value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// let blackboard = bevy_game::blackboard::Blackboard::new();
    /// ```
    pub fn new() -> Self {
        Self(serde_json::json!({}))
    }

    /// Gets a reference to the value corresponding to the key.
    ///
    /// If the key does not exist, this function will panic. It is recommended to ensure that the
    /// key exists before calling this method to avoid runtime panics.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the name of the key.
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
    /// This method allows you to store data in the `Blackboard` by associating a key with a value.
    /// The key must be a string, and the value can be any valid JSON value.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the name of the key.
    /// * `value` - A `serde_json::Value` that holds the value to be associated with the key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut blackboard = bevy_game::blackboard::Blackboard::new();
    /// blackboard.set("key", serde_json::json!("value"));
    /// assert_eq!(blackboard.get("key"), &serde_json::json!("value"));
    /// ```
    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        self.0
            .as_object_mut()
            .unwrap()
            .insert(key.to_string(), value);
    }
}

impl Default for Blackboard {
    fn default() -> Self {
        Self::new()
    }
}
