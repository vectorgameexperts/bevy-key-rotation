use async_trait::async_trait;
use bevy::{
    log::{self, LogPlugin},
    prelude::*,
};
use bevy_key_rotation::{
    AuthProvider, KeyRotationPlugin, KeyRotationSettings, Keystore,
    KeystoreState, TokenRotationError,
};
use std::{sync::Arc, time::Duration};

pub struct MyAuthProvider;

#[async_trait]
impl AuthProvider for MyAuthProvider {
    async fn authenticate(
        &self,
        username: String,
        password: String,
    ) -> Result<Keystore, TokenRotationError> {
        Ok(Keystore {
            username,
            password,
            access_token: "123".to_string(),
            refresh_token: "456".to_string(),
            access_expires: bevy_key_rotation::Instant::now()
                + Duration::from_secs(20),
            refresh_expires: bevy_key_rotation::Instant::now()
                + Duration::from_secs(60),
        })
    }
    async fn refresh(
        &self,
        _keystore: Keystore,
    ) -> Result<Keystore, TokenRotationError> {
        #[derive(thiserror::Error, Default, Debug)]
        #[error("This fails on purpose!")]
        struct MyError;
        Err(TokenRotationError::new(MyError))
    }
}

fn status_check(
    time: Res<Time>,
    mut update_every: Local<Option<Timer>>,
    keystore: Res<Keystore>,
) {
    // Print status every few seconds...
    const PRINT_EVERY_SECONDS: f32 = 1.0;
    let update_every = update_every.get_or_insert(Timer::from_seconds(
        PRINT_EVERY_SECONDS,
        TimerMode::Repeating,
    ));
    update_every.tick(time.delta());
    if !update_every.finished() {
        return;
    }

    if keystore.access_token_valid_for() < Duration::from_secs(5) {
        log::error!("The keystore is about to be non-conformant!");
        // You could attempt to re-authenticate from scratch by overriding the keystore:
        // (*keystore) = MyAuthProvider.authenticate(...);
        // Or react to this, perhaps safing your system and prepare to exit, etc.
    }

    // Log current access token
    log::info!(
        token = keystore.access_token,
        refresh_token = keystore.refresh_token,
        "token valid for: {:.0?}, refresh token valid for: {:.0?}",
        keystore.access_token_valid_for(),
        keystore.refresh_token_valid_for(),
    );
}

pub fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default()))
        .add_plugins(KeyRotationPlugin {
            username: "username".to_string(),
            password: "password".to_string(),
            rotation_settings: KeyRotationSettings {
                rotation_timeout: bevy_key_rotation::Duration::MAX, // no timeout
                rotation_check_interval: bevy_key_rotation::Duration::from_secs(
                    5,
                ),
                rotate_before: bevy_key_rotation::Duration::from_secs(15),
            },
            auth_provider: Arc::new(MyAuthProvider),
        })
        .add_systems(
            Update,
            status_check
                .run_if(state_exists_and_equals(KeystoreState::Conformant)),
        )
        .add_systems(OnEnter(KeystoreState::NonConformant), || {
            panic!("Keystore is now non-conformant! Keys cannot be updated.");
        })
        .run();
}