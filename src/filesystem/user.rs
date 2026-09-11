use std::collections::HashMap;

use crate::error::{EnvError, EnvResult};

pub const ROOT_USER_ID: u32 = 0;
pub const DEFAULT_USER_ID: u32 = 1000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: u32,
    pub username: String,
}

impl User {
    pub fn new(id: u32, username: impl Into<String>) -> EnvResult<Self> {
        let username = username.into();

        if username.is_empty() {
            return Err(EnvError::InvalidEnvironment(
                "Username cannot be empty.".to_string(),
            ));
        }

        Ok(Self { id, username })
    }
}

#[derive(Debug)]
pub struct UserManager {
    users: HashMap<u32, User>,
}

impl UserManager {
    pub fn new() -> Self {
        let mut manager = Self {
            users: HashMap::new(),
        };

        manager.users.insert(
            ROOT_USER_ID,
            User {
                id: ROOT_USER_ID,
                username: "root".to_string(),
            },
        );

        manager.users.insert(
            DEFAULT_USER_ID,
            User {
                id: DEFAULT_USER_ID,
                username: "user".to_string(),
            },
        );

        manager
    }

    pub fn add_user(&mut self, user: User) -> EnvResult<()> {
        if self.users.contains_key(&user.id) {
            return Err(EnvError::AlreadyExists(format!(
                "User ID {} already exists.",
                user.id
            )));
        }

        if self
            .users
            .values()
            .any(|existing| existing.username == user.username)
        {
            return Err(EnvError::AlreadyExists(format!(
                "Username '{}' already exists.",
                user.username
            )));
        }

        self.users.insert(user.id, user);

        Ok(())
    }

    pub fn get_user(&self, id: u32) -> Option<&User> {
        self.users.get(&id)
    }

    pub fn get_user_by_name(&self, username: &str) -> Option<&User> {
        self.users.values().find(|user| user.username == username)
    }

    pub fn contains(&self, id: u32) -> bool {
        self.users.contains_key(&id)
    }
}

impl Default for UserManager {
    fn default() -> Self {
        Self::new()
    }
}
