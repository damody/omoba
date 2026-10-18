//! Current visible channel only. No command, destination, origin or private timer.
pub const SCHEMA_ID: u32 = 0x464f4716;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChannelVisualState {
    #[default]
    None,
    Recall,
}

impl ChannelVisualState {
    pub fn encode(self) -> Vec<u8> {
        let mut bytes = b"CVS1".to_vec();
        bytes.push(match self { Self::None => 0, Self::Recall => 1 });
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 5 || &bytes[..4] != b"CVS1" { return None; }
        match bytes[4] { 0 => Some(Self::None), 1 => Some(Self::Recall), _ => None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_visual_state_codec_rejects_unknown_or_extended_private_payload() {
        assert_eq!(ChannelVisualState::decode(b"CVS1\0"), Some(ChannelVisualState::None));
        assert_eq!(ChannelVisualState::decode(b"CVS1\x01"), Some(ChannelVisualState::Recall));
        for state in [ChannelVisualState::None, ChannelVisualState::Recall] {
            assert_eq!(ChannelVisualState::decode(&state.encode()), Some(state));
        }
        for invalid in [b"CVS0\x01".as_slice(), b"CVS1\x02", b"CVS1", b"CVS1\x01\0", b""] {
            assert_eq!(ChannelVisualState::decode(invalid), None);
        }
        for unknown in 2..=255 { assert_eq!(ChannelVisualState::decode(&[b'C',b'V',b'S',b'1',unknown]), None); }
    }
}
