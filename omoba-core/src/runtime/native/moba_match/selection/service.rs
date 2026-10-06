//! Bounded JSON-line transport for a host-bound pre-match selection session.
//! The caller admits the identity; this transport is not authentication.
use super::*;
use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};

pub const SELECTION_PROTOCOL_VERSION: u32 = 1;
pub const MAX_SELECTION_LINE_BYTES: usize = 16 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionCommand {
    pub protocol_version: u32,
    pub catalog_data_hash: String,
    pub request_id: u64,
    pub expected_revision: u64,
    pub action: SelectionCommandAction,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SelectionCommandAction {
    Read,
    Select { hero: String },
    Lock,
    Finalize,
}

#[derive(Debug, Serialize)]
pub struct SelectionReply {
    pub protocol_version: u32,
    /// Exact decimal token for consumers whose JSON numbers use f64.
    pub revision_token: String,
    pub request_id_token: Option<String>,
    pub admitted_player_id: u32,
    pub request_id: Option<u64>,
    pub selection: HeroSelectionSnapshot,
    pub error: Option<String>,
    pub plan: Option<RoleBotMatchPlan>,
    pub hero_catalog: Vec<HeroSelectionOption>,
}

/// One host-owned selection authority, shared by all bound connections.
/// Cloning a room clones its handle, never its selection state or a World.
#[derive(Clone)]
pub struct SelectionRoom {
    state: Arc<Mutex<SelectionRoomState>>,
}

struct SelectionRoomState {
    session: HeroSelectionSession,
    finalized_plan: Option<RoleBotMatchPlan>,
}

impl SelectionRoom {
    pub fn new(session: HeroSelectionSession) -> Self {
        Self {
            state: Arc::new(Mutex::new(SelectionRoomState {
                session,
                finalized_plan: None,
            })),
        }
    }

    /// The host supplies an already admitted identity. This is not authentication.
    /// Commands on the returned connection cannot supply or change a player ID.
    pub fn bind(&self, admitted_player_id: u32) -> Result<SelectionService, String> {
        self.with_session(|session| {
            if !session
                .snapshot()
                .seats
                .iter()
                .any(|seat| seat.player.player_id == admitted_player_id && !seat.player.bot)
            {
                return Err("selection service requires an admitted human seat".into());
            }
            Ok(SelectionService {
                room: self.clone(),
                admitted_player_id,
            })
        })?
    }

    /// Host-only, one-time handoff. It survives the finalizing connection dropping.
    /// A reconnect/read may receive a read-only recipe, never recreate this
    /// host launch handoff or start a second match.
    pub fn take_finalized_plan(&self) -> Result<Option<RoleBotMatchPlan>, String> {
        self.with_state(|state| state.finalized_plan.take())
    }

    /// Read-only host view, independent of any connected player's request IDs.
    pub fn snapshot(&self) -> Result<HeroSelectionSnapshot, String> {
        self.with_session(|session| session.snapshot())
    }

    fn with_session<T>(
        &self,
        action: impl FnOnce(&mut HeroSelectionSession) -> T,
    ) -> Result<T, String> {
        self.with_state(|state| action(&mut state.session))
    }

    fn with_state<T>(
        &self,
        action: impl FnOnce(&mut SelectionRoomState) -> T,
    ) -> Result<T, String> {
        // Never recover a poisoned authority by accepting possibly partial state.
        let mut state = self
            .state
            .lock()
            .map_err(|_| "selection room authority is unavailable")?;
        Ok(action(&mut state))
    }
}

/// Persistent player-bound adapter reusable by CLI and renderer/lobby hosts.
pub struct SelectionService {
    room: SelectionRoom,
    admitted_player_id: u32,
}

impl SelectionService {
    pub fn new(session: HeroSelectionSession, admitted_player_id: u32) -> Result<Self, String> {
        SelectionRoom::new(session).bind(admitted_player_id)
    }

    fn reply(
        &self,
        session: &HeroSelectionSession,
        request_id: Option<u64>,
        error: Option<String>,
        plan: Option<RoleBotMatchPlan>,
    ) -> SelectionReply {
        SelectionReply {
            protocol_version: SELECTION_PROTOCOL_VERSION,
            revision_token: session.snapshot().revision.to_string(),
            request_id_token: request_id.map(|id| id.to_string()),
            admitted_player_id: self.admitted_player_id,
            request_id,
            selection: session.snapshot(),
            error,
            plan: plan.or_else(|| session.finalized_plan_snapshot()),
            hero_catalog: hero_selection_catalog(),
        }
    }

    pub fn handle(&mut self, command: SelectionCommand) -> Result<SelectionReply, String> {
        // Validation, mutation and the corresponding reply snapshot share one lock.
        // Concurrent connections cannot both commit the same expected revision.
        self.room.with_state(|state| {
            let result = (|| {
                let session = &mut state.session;
                if command.protocol_version != SELECTION_PROTOCOL_VERSION {
                    return Err("selection protocol version mismatch".into());
                }
                if command.catalog_data_hash != omoba_template_ids::CONTENT_CATALOG_DATA_HASH {
                    return Err("selection catalog hash mismatch".into());
                }
                if command.expected_revision != session.snapshot().revision {
                    return Err("stale hero selection revision".into());
                }
                let action = match command.action {
                    SelectionCommandAction::Read => return Ok(None),
                    SelectionCommandAction::Finalize => {
                        return session.finalize(command.expected_revision).map(Some)
                    }
                    SelectionCommandAction::Select { hero } => HeroSelectionAction::Select { hero },
                    SelectionCommandAction::Lock => HeroSelectionAction::Lock,
                };
                session.apply(
                    self.admitted_player_id,
                    HeroSelectionRequest {
                        expected_revision: command.expected_revision,
                        action,
                    },
                )?;
                Ok(None)
            })();
            if let Ok(Some(plan)) = &result {
                state.finalized_plan = Some(plan.clone());
            }
            match result {
                Ok(plan) => self.reply(&state.session, Some(command.request_id), None, plan),
                Err(error) => {
                    self.reply(&state.session, Some(command.request_id), Some(error), None)
                }
            }
        })
    }

    fn transport_reply(&self, error: Option<String>) -> io::Result<SelectionReply> {
        self.room
            .with_session(|session| self.reply(session, None, error, None))
            .map_err(io::Error::other)
    }

    /// Sends initial catalog/state, then one flushed reply per complete line.
    /// EOF never locks/finalizes. Oversized frames terminate instead of treating
    /// the remaining bytes as another command. An unterminated EOF is ignored.
    pub fn serve(&mut self, mut input: impl BufRead, mut output: impl Write) -> io::Result<()> {
        fn send(output: &mut impl Write, reply: &SelectionReply) -> io::Result<()> {
            serde_json::to_writer(&mut *output, reply)?;
            output.write_all(b"\n")?;
            output.flush()
        }
        send(&mut output, &self.transport_reply(None)?)?;
        loop {
            let mut line = Vec::new();
            loop {
                let available = input.fill_buf()?;
                if available.is_empty() {
                    return Ok(());
                }
                let count = available
                    .iter()
                    .position(|b| *b == b'\n')
                    .map_or(available.len(), |n| n + 1);
                if line.len() + count > MAX_SELECTION_LINE_BYTES {
                    send(
                        &mut output,
                        &self.transport_reply(Some("selection frame exceeds size limit".into()))?,
                    )?;
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "selection frame exceeds size limit",
                    ));
                }
                let complete = available[count - 1] == b'\n';
                line.extend_from_slice(&available[..count]);
                input.consume(count);
                if complete {
                    break;
                }
            }
            let reply = match serde_json::from_slice::<SelectionCommand>(&line) {
                Ok(command) => self.handle(command).map_err(io::Error::other)?,
                Err(_) => self.transport_reply(Some("invalid selection command JSON".into()))?,
            };
            send(&mut output, &reply)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot(service: &SelectionService) -> HeroSelectionSnapshot {
        service
            .room
            .with_session(|session| session.snapshot())
            .unwrap()
    }
    fn service() -> SelectionService {
        let mut plan = super::super::tests::plan();
        plan.players[1].bot = true;
        SelectionService::new(HeroSelectionSession::new(plan, 1, 60).unwrap(), 7).unwrap()
    }
    fn command(revision: u64, action: serde_json::Value) -> String {
        format!(
            "{}\n",
            serde_json::json!({"protocol_version":1,
            "catalog_data_hash":omoba_template_ids::CONTENT_CATALOG_DATA_HASH,
            "request_id":revision,"expected_revision":revision,"action":action})
        )
    }
    #[test]
    fn hero_selection_service_persistent_explicit_finalize() {
        let input = command(0, serde_json::json!({"kind":"read"}))
            + &command(
                0,
                serde_json::json!({"kind":"select","hero":"training_ranger"}),
            )
            + &command(1, serde_json::json!({"kind":"lock"}))
            + &command(2, serde_json::json!({"kind":"finalize"}));
        let mut output = Vec::new();
        service()
            .serve(io::Cursor::new(input), &mut output)
            .unwrap();
        let replies: Vec<serde_json::Value> = String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(replies.len(), 5);
        assert!(replies.iter().all(|r| r["error"].is_null()));
        assert_eq!(replies[4]["plan"]["players"][0]["hero"], "training_ranger");
        assert_eq!(replies[4]["selection"]["revision"], 3);
        assert_eq!(replies[4]["revision_token"], "3");
        assert_eq!(replies[4]["request_id_token"], "2");
        assert_eq!(replies[4]["selection"]["finalized"], true);
    }
    #[test]
    fn hero_selection_service_errors_atomic_and_eof_does_not_finalize() {
        let mut service = service();
        let baseline = serde_json::to_value(snapshot(&service)).unwrap();
        for modification in [
            serde_json::json!({"protocol_version":99}),
            serde_json::json!({"catalog_data_hash":"bad"}),
            serde_json::json!({"expected_revision":9}),
            serde_json::json!({"player_id":8}),
        ] {
            let mut value: serde_json::Value =
                serde_json::from_str(&command(0, serde_json::json!({"kind":"lock"}))).unwrap();
            for (key, val) in modification.as_object().unwrap() {
                value[key] = val.clone();
            }
            let mut output = Vec::new();
            service
                .serve(io::Cursor::new(format!("{value}\n")), &mut output)
                .unwrap();
            let last: serde_json::Value =
                serde_json::from_str(String::from_utf8(output).unwrap().lines().last().unwrap())
                    .unwrap();
            assert!(last["error"].is_string());
            assert_eq!(last["selection"], baseline);
        }
        service
            .serve(
                io::Cursor::new(command(0, serde_json::json!({"kind":"lock"})).trim_end()),
                Vec::new(),
            )
            .unwrap();
        assert_eq!(serde_json::to_value(snapshot(&service)).unwrap(), baseline);
        service
            .serve(
                io::Cursor::new(command(0, serde_json::json!({"kind":"lock"}))),
                Vec::new(),
            )
            .unwrap();
        assert!(snapshot(&service).ready);
        assert!(!snapshot(&service).finalized);
    }
    #[test]
    fn hero_selection_service_bounds_frames_and_validates_binding() {
        let mut service = service();
        assert!(service
            .serve(
                io::Cursor::new(vec![b'x'; MAX_SELECTION_LINE_BYTES + 1]),
                Vec::new()
            )
            .is_err());
        assert_eq!(snapshot(&service).revision, 0);
        for (id, rejected) in [(0, true), (8, false), (9, true)] {
            assert_eq!(
                SelectionService::new(
                    HeroSelectionSession::new(super::super::tests::plan(), 1, 60).unwrap(),
                    id,
                )
                .is_err(),
                rejected
            );
        }
    }

    #[test]
    fn hero_selection_room_two_bound_humans_share_selection_and_finalize() {
        let room = SelectionRoom::new(
            HeroSelectionSession::new(super::super::tests::plan(), 1, 60).unwrap(),
        );
        for id in [0, 9, 999] {
            assert!(room.bind(id).is_err());
        }
        let mut first = room.bind(7).unwrap();
        let mut second = room.bind(8).unwrap();
        assert!(first.transport_reply(None).unwrap().plan.is_none());
        assert!(room.take_finalized_plan().unwrap().is_none());
        let apply = |service: &mut SelectionService, revision, action| {
            service
                .handle(serde_json::from_str(&command(revision, action)).unwrap())
                .unwrap()
        };
        let selected = apply(
            &mut first,
            0,
            serde_json::json!({"kind":"select","hero":"training_ranger"}),
        );
        assert!(selected.error.is_none());
        assert_eq!(snapshot(&second).seats[0].player.hero, "training_ranger");
        let stale = apply(&mut second, 0, serde_json::json!({"kind":"lock"}));
        assert_eq!(
            stale.error.as_deref(),
            Some("stale hero selection revision")
        );
        assert_eq!(stale.admitted_player_id, 8);
        assert_eq!(stale.request_id_token.as_deref(), Some("0"));
        assert_eq!(stale.selection.revision, 1);
        assert!(!stale.selection.seats[1].locked);
        assert!(apply(&mut first, 1, serde_json::json!({"kind":"finalize"}))
            .error
            .is_some());
        assert!(apply(&mut first, 1, serde_json::json!({"kind":"lock"}))
            .error
            .is_none());
        assert!(apply(
            &mut second,
            2,
            serde_json::json!({"kind":"select","hero":"training_luminary"})
        )
        .error
        .is_none());
        assert!(apply(&mut second, 3, serde_json::json!({"kind":"lock"}))
            .error
            .is_none());
        assert!(second.transport_reply(None).unwrap().plan.is_none());
        let final_reply = apply(&mut first, 4, serde_json::json!({"kind":"finalize"}));
        let plan = final_reply.plan.unwrap();
        drop(first); // Host handoff does not depend on the finalizing connection.
        let handed_off = room.take_finalized_plan().unwrap().unwrap();
        assert_eq!(
            serde_json::to_value(&handed_off).unwrap(),
            serde_json::to_value(&plan).unwrap()
        );
        assert!(room.clone().take_finalized_plan().unwrap().is_none());
        assert!(final_reply.selection.finalized && final_reply.selection.ready);
        assert_eq!(plan.players[0].hero, "training_ranger");
        assert_eq!(plan.players[1].hero, "training_luminary");
        assert_eq!(snapshot(&second).revision, 5);
        let frozen = serde_json::to_value(snapshot(&second)).unwrap();
        assert!(apply(
            &mut second,
            5,
            serde_json::json!({"kind":"select","hero":"training_ranger"})
        )
        .error
        .is_some());
        assert_eq!(serde_json::to_value(snapshot(&second)).unwrap(), frozen);
        assert!(room.take_finalized_plan().unwrap().is_none());
        // Rebinding/EOF only returns the shared state, never unlocks or finalizes.
        let mut reconnected = room.clone().bind(8).unwrap();
        let mut output = Vec::new();
        reconnected
            .serve(io::Cursor::new(Vec::<u8>::new()), &mut output)
            .unwrap();
        let reply: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(reply["selection"], frozen);
        assert_eq!(reply["admitted_player_id"], 8);
        assert!(reply["request_id_token"].is_null());
        assert_eq!(reply["plan"], serde_json::to_value(&plan).unwrap());
        assert!(room.take_finalized_plan().unwrap().is_none());
        // A stale follower read receives the same immutable final recipe without
        // recreating the launch authority or changing the finalized revision.
        let stale_read = apply(&mut second, 4, serde_json::json!({"kind":"read"}));
        assert!(stale_read.error.is_some());
        assert_eq!(
            serde_json::to_value(stale_read.plan.unwrap()).unwrap(),
            reply["plan"]
        );
        assert_eq!(serde_json::to_value(snapshot(&second)).unwrap(), frozen);
        assert!(room.take_finalized_plan().unwrap().is_none());
    }

    #[test]
    fn hero_selection_room_concurrent_revision_commits_once() {
        let room = SelectionRoom::new(
            HeroSelectionSession::new(super::super::tests::plan(), 1, 60).unwrap(),
        );
        let barrier = Arc::new(std::sync::Barrier::new(3));
        let mut threads = Vec::new();
        for id in [7, 8] {
            let mut connection = room.bind(id).unwrap();
            let barrier = barrier.clone();
            threads.push(std::thread::spawn(move || {
                barrier.wait();
                connection
                    .handle(
                        serde_json::from_str(&command(0, serde_json::json!({"kind":"lock"})))
                            .unwrap(),
                    )
                    .unwrap()
            }));
        }
        barrier.wait();
        let replies: Vec<_> = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect();
        assert_eq!(
            replies.iter().filter(|reply| reply.error.is_none()).count(),
            1
        );
        assert!(replies.iter().all(|reply| reply.selection.revision == 1));
        let loser = replies
            .iter()
            .find(|reply| reply.error.is_some())
            .unwrap()
            .admitted_player_id;
        let mut connection = room.bind(loser).unwrap();
        assert_eq!(
            snapshot(&connection)
                .seats
                .iter()
                .filter(|seat| seat.locked && !seat.player.bot)
                .count(),
            1
        );
        let next = connection
            .handle(serde_json::from_str(&command(1, serde_json::json!({"kind":"lock"}))).unwrap())
            .unwrap();
        assert!(next.error.is_none() && next.selection.ready);
        assert_eq!(next.selection.revision, 2);
        assert!(!next.selection.finalized);
        // Competing finalize requests still produce exactly one host handoff.
        let barrier = Arc::new(std::sync::Barrier::new(3));
        let mut threads = Vec::new();
        for id in [7, 8] {
            let mut connection = room.bind(id).unwrap();
            let barrier = barrier.clone();
            threads.push(std::thread::spawn(move || {
                barrier.wait();
                connection
                    .handle(
                        serde_json::from_str(&command(2, serde_json::json!({"kind":"finalize"})))
                            .unwrap(),
                    )
                    .unwrap()
            }));
        }
        barrier.wait();
        let replies: Vec<_> = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect();
        assert_eq!(
            replies.iter().filter(|reply| reply.error.is_none()).count(),
            1
        );
        assert!(replies.iter().all(|reply| reply.plan.is_some()));
        assert_eq!(
            serde_json::to_value(replies[0].plan.as_ref().unwrap()).unwrap(),
            serde_json::to_value(replies[1].plan.as_ref().unwrap()).unwrap()
        );
        assert!(room.take_finalized_plan().unwrap().is_some());
        assert!(room.take_finalized_plan().unwrap().is_none());
    }

    #[test]
    fn hero_selection_room_poison_fails_closed_without_transport_output() {
        let room = SelectionRoom::new(
            HeroSelectionSession::new(super::super::tests::plan(), 1, 60).unwrap(),
        );
        let mut connection = room.bind(7).unwrap();
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = room.with_session(|_| panic!("injected authority failure"));
        }));
        assert!(failure.is_err());
        assert!(room.bind(8).is_err());
        assert!(room.take_finalized_plan().is_err());
        let mut output = Vec::new();
        assert!(connection
            .serve(io::Cursor::new(Vec::<u8>::new()), &mut output)
            .is_err());
        assert!(output.is_empty());
        assert!(connection
            .handle(serde_json::from_str(&command(0, serde_json::json!({"kind":"lock"}))).unwrap())
            .is_err());
    }

    #[test]
    fn hero_selection_service_recovers_json_errors_but_never_locks_other_humans() {
        let mut service = SelectionService::new(
            HeroSelectionSession::new(super::super::tests::plan(), 1, 60).unwrap(),
            7,
        )
        .unwrap();
        let input = "not-json\n".to_string()
            + &command(0, serde_json::json!({"kind":"lock"}))
            + &command(1, serde_json::json!({"kind":"finalize"}));
        let mut output = Vec::new();
        service.serve(io::Cursor::new(input), &mut output).unwrap();
        let replies: Vec<serde_json::Value> = String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert!(replies[1]["error"].is_string());
        assert!(replies[2]["error"].is_null());
        assert!(replies[3]["error"].is_string());
        assert_eq!(replies[3]["selection"]["revision"], 1);
        assert_eq!(replies[3]["selection"]["seats"][1]["locked"], false);
        assert!(replies[3]["plan"].is_null());
    }
}
