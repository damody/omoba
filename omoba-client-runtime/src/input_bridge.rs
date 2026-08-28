use omoba_core::game_proto::{
    player_input, renderer_input, AttackMove, CastAbility, ItemUse, PlayerInput, RendererInput,
    TowerPlace, TowerSell, TowerUpgradeInput, Vec2I,
};

use crate::replica_host::ReplicaHost;

#[derive(Debug)]
pub enum InputDecision {
    Accepted {
        input_id: u32,
        input: PlayerInput,
        secure_target: Option<omoba_core::game_proto::SecureReplicaTarget>,
    },
    Rejected {
        request_id: u64,
        code: &'static str,
    },
}

#[derive(Default)]
pub struct InputBridge {
    next_input_id: u32,
}

impl InputBridge {
    pub fn allocate_input_id(&mut self) -> u32 {
        self.next_input_id = self.next_input_id.wrapping_add(1).max(1);
        self.next_input_id
    }

    pub fn validate(
        &mut self,
        renderer_input: RendererInput,
        configured_player_id: u32,
        replica: &ReplicaHost,
    ) -> InputDecision {
        if renderer_input.player_id != configured_player_id {
            return reject(renderer_input.request_id, "INVALID_OWNER");
        }
        if renderer_input.disclosure_epoch != replica.view_epoch() {
            return reject(renderer_input.request_id, "STALE_DISCLOSURE_EPOCH");
        }
        if !replica.owns_hero(configured_player_id) {
            return reject(renderer_input.request_id, "OWN_HERO_NOT_DISCLOSED");
        }
        let mut secure_target = None;
        let action = match renderer_input.intent {
            Some(renderer_input::Intent::MoveTo(intent)) => {
                player_input::Action::MoveTo(omoba_core::game_proto::MoveTo {
                    target: fixed_vec(intent.x_raw, intent.y_raw),
                    queued: false,
                })
            }
            Some(renderer_input::Intent::AttackMove(intent)) => {
                player_input::Action::AttackMove(AttackMove {
                    target: fixed_vec(intent.x_raw, intent.y_raw),
                    queued: false,
                })
            }
            Some(renderer_input::Intent::AbilityCast(intent)) => {
                let target_entity = match optional_target(
                    intent.target_render_id,
                    renderer_input.disclosure_epoch,
                    replica,
                ) {
                    Ok(value) => value,
                    Err(code) => return reject(renderer_input.request_id, code),
                };
                secure_target = (intent.target_render_id != 0)
                    .then(|| replica.secure_reference(intent.target_render_id))
                    .flatten();
                player_input::Action::CastAbility(CastAbility {
                    ability_index: intent.ability_index,
                    target_pos: fixed_vec(intent.x_raw, intent.y_raw),
                    target_entity,
                })
            }
            Some(renderer_input::Intent::ItemUse(intent)) => {
                let target_entity = match optional_target(
                    intent.target_render_id,
                    renderer_input.disclosure_epoch,
                    replica,
                ) {
                    Ok(value) => value,
                    Err(code) => return reject(renderer_input.request_id, code),
                };
                secure_target = (intent.target_render_id != 0)
                    .then(|| replica.secure_reference(intent.target_render_id))
                    .flatten();
                player_input::Action::ItemUse(ItemUse {
                    item_slot: intent.item_slot,
                    target_pos: fixed_vec(intent.x_raw, intent.y_raw),
                    target_entity,
                })
            }
            Some(renderer_input::Intent::TowerAction(intent)) => match intent.action_kind {
                1 => player_input::Action::TowerPlace(TowerPlace {
                    tower_kind_id: intent.tower_kind_id,
                    pos: fixed_vec(intent.x_raw, intent.y_raw),
                }),
                2 => {
                    let Ok(tower_entity_id) = u32::try_from(intent.tower_render_id) else {
                        return reject(renderer_input.request_id, "INVALID_TARGET");
                    };
                    if replica.secure_reference(intent.tower_render_id).is_none() {
                        return reject(renderer_input.request_id, "INVALID_TARGET");
                    }
                    secure_target = replica.secure_reference(intent.tower_render_id);
                    player_input::Action::TowerUpgrade(TowerUpgradeInput {
                        tower_entity_id,
                        path: intent.path,
                        level: intent.level,
                    })
                }
                3 => {
                    let Ok(tower_entity_id) = u32::try_from(intent.tower_render_id) else {
                        return reject(renderer_input.request_id, "INVALID_TARGET");
                    };
                    if replica.secure_reference(intent.tower_render_id).is_none() {
                        return reject(renderer_input.request_id, "INVALID_TARGET");
                    }
                    secure_target = replica.secure_reference(intent.tower_render_id);
                    player_input::Action::TowerSell(TowerSell { tower_entity_id })
                }
                _ => return reject(renderer_input.request_id, "INVALID_ACTION"),
            },
            None => return reject(renderer_input.request_id, "MISSING_ACTION"),
        };
        InputDecision::Accepted {
            input_id: self.allocate_input_id(),
            input: PlayerInput {
                action: Some(action),
            },
            secure_target,
        }
    }
}

fn fixed_vec(x_raw: i64, y_raw: i64) -> Option<Vec2I> {
    Some(Vec2I {
        x: x_raw.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        y: y_raw.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
    })
}

fn optional_target(
    render_id: u64,
    disclosure_epoch: u64,
    replica: &ReplicaHost,
) -> Result<Option<u32>, &'static str> {
    if render_id == 0 {
        return Ok(None);
    }
    let _ = disclosure_epoch;
    if replica.secure_reference(render_id).is_none() {
        return Err("INVALID_TARGET");
    }
    u32::try_from(render_id)
        .map(Some)
        .map_err(|_| "INVALID_TARGET")
}

fn reject(request_id: u64, code: &'static str) -> InputDecision {
    InputDecision::Rejected { request_id, code }
}
