macro_rules! declare_id_newtype {
    ($(#[$outer:meta])* $name:ident) => {
        $(#[$outer])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct $name(uuid::Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
        }

        impl std::ops::Deref for $name {
            type Target = uuid::Uuid;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }


        impl From<uuid::Uuid> for $name {
            fn from(value: uuid::Uuid) -> Self {
                Self(value)
            }
        }

        impl Into<uuid::Uuid> for $name {
            fn into(self) -> uuid::Uuid {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_hyphenated())
            }
        }
    };
}

pub mod readonly {
    use std::sync::{Arc, RwLock, RwLockReadGuard};
    pub struct ReadOnly<T>(Arc<RwLock<T>>);

    impl<T> Clone for ReadOnly<T> {
        fn clone(&self) -> Self {
            ReadOnly(Arc::clone(&self.0))
        }
    }

    impl<T> ReadOnly<T> {
        pub fn new(value: Arc<RwLock<T>>) -> Self {
            Self(value)
        }

        pub fn read(&self) -> RwLockReadGuard<'_, T> {
            self.0.read().unwrap()
        }
    }
}

#[derive(Clone, Copy)]
pub enum DocEvent {
    UniverseSettingsChanged,
}

pub trait DocObserver {
    fn on_doc_event(&mut self, event: DocEvent);
}

pub mod engine;
pub mod fixture;
pub mod functions;
pub mod plugins;
//pub mod qxw_loader;
pub mod commands;
pub mod doc;
pub mod fixture_def;
pub mod universe;
