//! Authority-owned input scheme and bounded held-direction state.
//! No OS clock, renderer state, movement mutation or untrusted expiry is accepted.
#![forbid(unsafe_code)]
use omoba_sim::{Fixed64, Vec2};
use super::selective::CanonicalEntityKey;

/// Rebuilt by the command system for one exact simulation tick, never persisted or transmitted.
#[derive(Default)]
pub(crate) struct DirectionalStepPlan {
    pub tick: u64,
    pub directions: std::collections::BTreeMap<specs::Entity, Vec2>,
}

/// Authority-only pre-dispatch controls. Unlike the post-step roster lease,
/// these survive a Stop or point command drained after movement in this tick.
/// Projectors must translate the exact private life into their disclosed namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OwnerDirectionalStep {
    pub source: specs::Entity,
    pub scheme: InputScheme,
    pub direction: Vec2,
    pub press_id: u64,
    pub navigation_reset: bool,
    pub attack_reset: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InputSchemeStepSnapshot {
    pub tick: u64,
    pub rows: std::collections::BTreeMap<u32, OwnerDirectionalStep>,
}

/// Exact-tick admitted controls on already disclosed local ECS bodies only.
#[derive(Default)]
pub(crate) struct AdmittedDirectionalSteps {
    pub tick: u64,
    pub rows: std::collections::BTreeMap<specs::Entity, super::input_scheme_transport::AdmittedDirectionStep>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum InputScheme {
    #[default]
    PointAndClick = 0,
    Keyboard = 1,
}
impl InputScheme {
    pub fn from_raw(raw: u32) -> Result<Self, InputSchemeError> {
        match raw { 0 => Ok(Self::PointAndClick), 1 => Ok(Self::Keyboard), _ => Err(InputSchemeError::InvalidScheme) }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputSchemeError { WrongOwner, InvalidPhase, InvalidScheme, Locked, InvalidClock, InvalidRules, InvalidAxis, InvalidSource, InvalidGesture, RetiredGesture }

/// The owner supplies a monotonically increasing press id for the current hero
/// life. An update cannot resurrect a press retired by Stop, recall or focus loss.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectionGesture { Begin(u64), Update(u64), End(u64) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectionAdmission { Began, Updated, Ended }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputSchemeRules { lock_after: Fixed64, direction_lease: Fixed64 }
impl Default for InputSchemeRules {
    fn default() -> Self { Self { lock_after: Fixed64::from_i32(600), direction_lease: Fixed64::from_raw(307) } }
}
impl InputSchemeRules {
    pub fn new(lock_after: Fixed64, direction_lease: Fixed64) -> Result<Self, InputSchemeError> {
        if lock_after < Fixed64::ZERO || lock_after > Fixed64::from_i32(3600)
            || !(102..=1024).contains(&direction_lease.raw()) { return Err(InputSchemeError::InvalidRules); }
        Ok(Self {lock_after, direction_lease})
    }
    pub fn lock_after(self) -> Fixed64 { self.lock_after }
    pub fn direction_lease(self) -> Fixed64 { self.direction_lease }
}

/// Normalizes digital or mapped joystick axes without floating point or tiny-vector amplification.
pub fn normalized_axes(x: i32, y: i32) -> Result<Vec2, InputSchemeError> {
    if !(-32767..=32767).contains(&x) || !(-32767..=32767).contains(&y) { return Err(InputSchemeError::InvalidAxis); }
    let x=i64::from(x);let y=i64::from(y);
    // Extra integer precision keeps (1,1) and full-scale digital diagonals identical.
    const SCALE: u128=1<<30;
    let square=((x*x+y*y) as u128)*SCALE*SCALE;
    if square==0 { return Ok(Vec2::ZERO); }
    let floor=square.isqrt();let ceil=floor+u128::from(floor*floor<square);
    Ok(Vec2::new(Fixed64::from_raw((i128::from(x)*1024*SCALE as i128/ceil as i128) as i64),Fixed64::from_raw((i128::from(y)*1024*SCALE as i128/ceil as i128) as i64)))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DirectionLease { source: CanonicalEntityKey, gesture: u64, direction: Vec2, starts_at: Fixed64, expires_at: Fixed64 }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OwnerInputScheme {
    scheme: InputScheme,
    held: Option<DirectionLease>,
    last_press: Option<(CanonicalEntityKey, u64)>,
}
impl OwnerInputScheme {
    pub fn scheme(self) -> InputScheme { self.scheme }
    pub fn locked(self, elapsed: Fixed64, rules: InputSchemeRules) -> Result<bool, InputSchemeError> {
        if elapsed < Fixed64::ZERO { return Err(InputSchemeError::InvalidClock); }
        Ok(elapsed>=rules.lock_after)
    }
    pub fn select(&mut self, raw: u32, elapsed: Fixed64, rules: InputSchemeRules) -> Result<(), InputSchemeError> {
        let next=InputScheme::from_raw(raw)?;
        let locked=self.locked(elapsed,rules)?;
        if next==self.scheme { return Ok(()); }
        if locked { return Err(InputSchemeError::Locked); }
        self.scheme=next;self.neutralize();Ok(())
    }
    pub fn last_press(self, source: CanonicalEntityKey) -> u64 {
        self.last_press.filter(|(life,_)| *life==source).map_or(0, |(_,id)| id)
    }
    pub fn active_press(self, source: CanonicalEntityKey) -> u64 {
        self.held.filter(|held| held.source==source).map_or(0,|held|held.gesture)
    }
    pub fn admit_direction(&mut self, source: CanonicalEntityKey, gesture: DirectionGesture,
        x: i32, y: i32, elapsed: Fixed64, rules: InputSchemeRules) -> Result<DirectionAdmission, InputSchemeError> {
        if self.scheme!=InputScheme::Keyboard { return Err(InputSchemeError::InvalidScheme); }
        if source.generation==0 || source.generation>i32::MAX as u32 { return Err(InputSchemeError::InvalidSource); }
        if elapsed<Fixed64::ZERO { return Err(InputSchemeError::InvalidClock); }
        let direction=normalized_axes(x,y)?;
        let expires_at=Fixed64::from_raw(elapsed.raw().checked_add(rules.direction_lease.raw()).ok_or(InputSchemeError::InvalidClock)?);
        let id = match gesture { DirectionGesture::Begin(id)|DirectionGesture::Update(id)|DirectionGesture::End(id)=>id };
        if id==0 || id==u64::MAX { return Err(InputSchemeError::InvalidGesture); }
        match gesture {
            DirectionGesture::Begin(_) => {
                if direction==Vec2::ZERO { return Err(InputSchemeError::InvalidGesture); }
                if id<=self.last_press(source) { return Err(InputSchemeError::RetiredGesture); }
                self.last_press=Some((source,id));
                self.held=Some(DirectionLease{source,gesture:id,direction,starts_at:elapsed,expires_at});
                Ok(DirectionAdmission::Began)
            }
            DirectionGesture::Update(_) => {
                if direction==Vec2::ZERO { return Err(InputSchemeError::InvalidGesture); }
                if !self.held.is_some_and(|held| held.source==source && held.gesture==id) {
                    return Err(InputSchemeError::RetiredGesture);
                }
                self.held=Some(DirectionLease{source,gesture:id,direction,starts_at:elapsed,expires_at});
                Ok(DirectionAdmission::Updated)
            }
            DirectionGesture::End(_) => {
                if direction!=Vec2::ZERO { return Err(InputSchemeError::InvalidGesture); }
                if self.last_press(source)!=id { return Err(InputSchemeError::RetiredGesture); }
                self.neutralize();
                Ok(DirectionAdmission::Ended)
            }
        }
    }
    pub fn direction(self, source: CanonicalEntityKey, elapsed: Fixed64) -> Vec2 {
        self.held.filter(|held| self.scheme==InputScheme::Keyboard && held.source==source && elapsed>=held.starts_at && elapsed<held.expires_at)
            .map_or(Vec2::ZERO,|held|held.direction)
    }
    pub fn neutralize(&mut self) { self.held=None; }
    /// Stable state material; a serializer cannot create an invalid admitted value.
    pub fn append_material(self, bytes: &mut Vec<u8>) {
        bytes.push(self.scheme as u8);bytes.push(u8::from(self.held.is_some()));
        bytes.push(u8::from(self.last_press.is_some()));
        if let Some((source,id))=self.last_press {
            bytes.extend(source.id.to_le_bytes());bytes.extend(source.generation.to_le_bytes());bytes.extend(id.to_le_bytes());
        }
        if let Some(held)=self.held {
            bytes.extend(held.source.id.to_le_bytes());bytes.extend(held.source.generation.to_le_bytes());
            bytes.extend(held.gesture.to_le_bytes());
            bytes.extend(held.direction.x.raw().to_le_bytes());bytes.extend(held.direction.y.raw().to_le_bytes());bytes.extend(held.starts_at.raw().to_le_bytes());bytes.extend(held.expires_at.raw().to_le_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_scheme_axis_norm_never_speeds_up_diagonals_or_tiny_inputs() {
        for x in [-32767,-500,-1,0,1,500,32767] { for y in [-32767,-1,0,1,32767] {
            let d=normalized_axes(x,y).unwrap();let square=d.x.raw()*d.x.raw()+d.y.raw()*d.y.raw();
            assert!(square<=1024*1024);if x!=0 || y!=0 { assert!(square>=1020*1020); }
        }}
        assert_eq!(normalized_axes(1,1),normalized_axes(32767,32767));
        assert_eq!(normalized_axes(0,1).unwrap(),Vec2::new(Fixed64::ZERO,Fixed64::ONE));
        assert_eq!(normalized_axes(i32::MIN,0),Err(InputSchemeError::InvalidAxis));
        assert_eq!(normalized_axes(0,32768),Err(InputSchemeError::InvalidAxis));
    }
    #[test]
    fn input_scheme_lock_is_authority_clock_and_invalid_update_is_atomic() {
        let mut state=OwnerInputScheme::default();let rules=InputSchemeRules::default();
        state.select(1,Fixed64::from_raw(600*1024-1),rules).unwrap();
        let before=state;
        assert_eq!(state.select(0,Fixed64::from_i32(600),rules),Err(InputSchemeError::Locked));assert_eq!(state,before);
        state.select(1,Fixed64::from_i32(1000),rules).unwrap();assert_eq!(state,before);
        assert_eq!(state.select(256,Fixed64::ZERO,rules),Err(InputSchemeError::InvalidScheme));assert_eq!(state,before);
        assert_eq!(state.select(0,Fixed64::from_raw(-1),rules),Err(InputSchemeError::InvalidClock));assert_eq!(state,before);
        assert!(InputSchemeRules::new(Fixed64::from_raw(-1),Fixed64::ONE).is_err());
        assert!(InputSchemeRules::new(Fixed64::ZERO,Fixed64::from_raw(101)).is_err());
    }
    #[test]
    fn input_scheme_direction_lease_is_source_scoped_bounded_and_neutralized() {
        let mut state=OwnerInputScheme::default();let rules=InputSchemeRules::default();
        let source=CanonicalEntityKey{id:0,generation:1};let now=Fixed64::from_i32(4);
        assert_eq!(state.admit_direction(source,DirectionGesture::Begin(1),1,0,now,rules),Err(InputSchemeError::InvalidScheme));
        state.select(1,now,rules).unwrap();state.admit_direction(source,DirectionGesture::Begin(1),1,0,now,rules).unwrap();let before=state;
        assert_eq!(state.direction(source,now),Vec2::new(Fixed64::ONE,Fixed64::ZERO));
        assert_eq!(state.direction(CanonicalEntityKey{generation:2,..source},now),Vec2::ZERO);
        assert_eq!(state.direction(source,now+rules.direction_lease()),Vec2::ZERO);
        assert_eq!(state.admit_direction(source,DirectionGesture::Update(1),32768,0,now,rules),Err(InputSchemeError::InvalidAxis));assert_eq!(state,before);
        assert_eq!(state.admit_direction(source,DirectionGesture::Update(1),1,0,Fixed64::from_raw(i64::MAX),rules),Err(InputSchemeError::InvalidClock));assert_eq!(state,before);
        state.admit_direction(source,DirectionGesture::End(1),0,0,now,rules).unwrap();assert_eq!(state.direction(source,now),Vec2::ZERO);
        state.admit_direction(source,DirectionGesture::Begin(2),0,1,now,rules).unwrap();state.select(0,now,rules).unwrap();state.select(1,now,rules).unwrap();assert_eq!(state.direction(source,now),Vec2::ZERO);
    }
    #[test]
    fn input_scheme_gesture_retirement_rejects_late_heartbeats_and_old_releases() {
        let mut state=OwnerInputScheme::default();let rules=InputSchemeRules::default();
        let life=CanonicalEntityKey{id:4,generation:1};let now=Fixed64::ONE;
        state.select(1,now,rules).unwrap();
        for id in [0,u64::MAX] {
            assert_eq!(state.admit_direction(life,DirectionGesture::Begin(id),1,0,now,rules),Err(InputSchemeError::InvalidGesture));
        }
        assert_eq!(state.last_press(life),0);
        state.admit_direction(life,DirectionGesture::Begin(1),1,0,now,rules).unwrap();
        let expired=now+rules.direction_lease();
        assert_eq!(state.direction(life,expired),Vec2::ZERO);
        assert_eq!(state.admit_direction(life,DirectionGesture::Update(1),1,0,expired,rules),Ok(DirectionAdmission::Updated));
        state.neutralize();let retired=state;
        assert_eq!(state.admit_direction(life,DirectionGesture::Update(1),1,0,expired,rules),Err(InputSchemeError::RetiredGesture));assert_eq!(state,retired);
        assert_eq!(state.admit_direction(life,DirectionGesture::Begin(1),1,0,expired,rules),Err(InputSchemeError::RetiredGesture));assert_eq!(state,retired);
        state.admit_direction(life,DirectionGesture::Begin(2),0,1,expired,rules).unwrap();let current=state;
        assert_eq!(state.admit_direction(life,DirectionGesture::End(1),0,0,expired,rules),Err(InputSchemeError::RetiredGesture));assert_eq!(state,current);
        assert_eq!(state.admit_direction(life,DirectionGesture::End(2),1,0,expired,rules),Err(InputSchemeError::InvalidGesture));assert_eq!(state,current);
        state.admit_direction(life,DirectionGesture::End(2),0,0,expired,rules).unwrap();
        state.admit_direction(life,DirectionGesture::End(2),0,0,expired,rules).unwrap();
        assert_eq!(state.direction(life,expired),Vec2::ZERO);
        let next_life=CanonicalEntityKey{generation:2,..life};
        assert_eq!(state.last_press(next_life),0);
        state.admit_direction(next_life,DirectionGesture::Begin(1),1,0,expired,rules).unwrap();
        assert_eq!(state.last_press(next_life),1);
    }
}
