// This file is part of Gear.

// Copyright (C) 2023 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::ProgressSignal;
use gstd::{msg, prelude::*, sync::RwLock, ActorId, MessageId};

#[derive(Debug)]
struct Session {
    init_message: MessageId,
    data: SessionData,
}

#[derive(Debug, Clone)]
pub struct SessionData {
    control_bus: ActorId,
    under_test_actor: ActorId,
}

impl SessionData {
    pub fn testee(&self) -> ActorId {
        self.under_test_actor.clone()
    }

    fn send_progress(&self, msg: ProgressSignal) {
        let _ = msg::send(self.control_bus, msg, 0);
    }

    pub fn test_start(&self, index: u32, name: &str) {
        gstd::debug!("test starts: {}", name);
        self.send_progress(ProgressSignal::new(index, name.to_string()));
    }

    pub fn test_success(&self, index: u32, name: &str) {
        gstd::debug!("test success: {}", name);
        self.send_progress(ProgressSignal::new(index, name.to_string()).success());
    }

    pub fn test_fail(&self, index: u32, name: &str, hint: String) {
        gstd::debug!("test fail: {}", name);
        self.send_progress(ProgressSignal::new(index, name.to_string()).fail(hint))
    }
}

// Vec is good enough if not much simultaneous sessions
static SESSIONS: RwLock<Vec<Session>> = RwLock::new(Vec::new());
static mut ACTIVE_SESSION: Option<SessionData> = None;

pub async fn new_session(under_test_actor: ActorId) -> (MessageId, SessionData) {
    let data = SessionData {
        control_bus: msg::source(),
        under_test_actor,
    };
    let init_message = msg::id();
    SESSIONS.write().await.push(Session {
        init_message: init_message.clone(),
        data: data.clone(),
    });

    (init_message, data)
}

/// Locate existing session.
///
/// If section is not found, panics.
pub async fn locate_session(init_message: &MessageId) -> SessionData {
    let sessions = SESSIONS.read().await;

    let found_session = sessions
        .iter()
        .find(|session| &session.init_message == init_message)
        .expect("Session not found. Terminating");

    found_session.data.clone()
}

pub async fn drop_session(init_message: &MessageId) {
    let mut sessions = SESSIONS.write().await;

    let found_index = sessions
        .iter()
        .position(|session| &session.init_message == init_message)
        .expect("Session not found. Terminating");

    sessions.swap_remove(found_index);
}

pub fn active_session() -> SessionData {
    unsafe {
        ACTIVE_SESSION
            .as_ref()
            .expect("Failed to find active session")
            .clone()
    }
}

pub(crate) async fn set_active_session(init_message: &MessageId) {
    let session_data = locate_session(init_message).await;
    unsafe {
        ACTIVE_SESSION = Some(session_data);
    }
}
