#[derive(Default)]
pub struct BleMidiDecoder {
    sysex: bool,
}

impl BleMidiDecoder {
    pub fn receive(&mut self, packet: &[u8], mut emit: impl FnMut(&[u8])) {
        if packet.len() < 2 || packet[0] & 0xc0 != 0x80 {
            return;
        }
        let mut bytes = packet[1..].iter().copied().peekable();
        let mut running = None;
        while let Some(mut byte) = bytes.next() {
            if byte >= 0x80 {
                // BLE-MIDI prefixes each explicit status with timestampLow.
                let Some(next) = bytes.next() else { return };
                byte = next;
            }
            if byte >= 0xf8 {
                continue;
            }
            if byte == 0xf0 {
                self.sysex = true;
                running = None;
                continue;
            }
            if byte == 0xf7 {
                self.sysex = false;
                running = None;
                continue;
            }
            if self.sysex {
                continue;
            }
            let (status, first) = if byte < 0x80 {
                let Some(status) = running else { return };
                (status, Some(byte))
            } else {
                if byte < 0xf0 {
                    running = Some(byte);
                }
                (byte, None)
            };
            let count = match status {
                0xc0..=0xdf | 0xf1 | 0xf3 => 1,
                0x80..=0xbf | 0xe0..=0xef | 0xf2 => 2,
                _ => 0,
            };
            let mut message = [status, 0, 0];
            for (i, value) in message[1..1 + count].iter_mut().enumerate() {
                if i == 0 && first.is_some() {
                    *value = byte;
                    continue;
                }
                loop {
                    match bytes.next() {
                        Some(data) if data < 0x80 => {
                            *value = data;
                            break;
                        }
                        Some(_) if bytes.peek().is_some_and(|next| *next >= 0xf8) => {
                            bytes.next();
                        }
                        _ => return,
                    }
                }
            }
            if status < 0xf0 {
                emit(&message[..=count]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BleMidiDecoder;

    fn decoded(decoder: &mut BleMidiDecoder, bytes: &[u8]) -> Vec<Vec<u8>> {
        let mut messages = Vec::new();
        decoder.receive(bytes, |message| messages.push(message.to_vec()));
        messages
    }

    #[test]
    fn removes_timestamps_and_respects_running_status() {
        let mut decoder = BleMidiDecoder::default();
        assert_eq!(
            decoded(
                &mut decoder,
                &[0x80, 0x81, 0x90, 36, 100, 37, 127, 0x82, 38, 99, 0x83, 0xc0, 7]
            ),
            [
                vec![0x90, 36, 100],
                vec![0x90, 37, 127],
                vec![0x90, 38, 99],
                vec![0xc0, 7]
            ]
        );
    }

    #[test]
    fn skips_sysex_continuation_and_realtime() {
        let mut decoder = BleMidiDecoder::default();
        assert!(decoded(&mut decoder, &[0x80, 0x81, 0xf0, 1, 2]).is_empty());
        assert_eq!(
            decoded(
                &mut decoder,
                &[0x80, 3, 0x82, 0xf8, 4, 0x83, 0xf7, 0x84, 0x90, 36, 0x85, 0xf8, 100]
            ),
            [vec![0x90, 36, 100]]
        );
    }

    #[test]
    fn system_common_preserves_running_status_with_new_timestamp() {
        let mut decoder = BleMidiDecoder::default();
        assert_eq!(
            decoded(
                &mut decoder,
                &[0x80, 0x81, 0xb0, 10, 64, 0x82, 0xf2, 1, 2, 0x83, 11, 127]
            ),
            [vec![0xb0, 10, 64], vec![0xb0, 11, 127]]
        );
    }

    #[test]
    fn ignores_truncated_messages_without_leaking_status_to_next_packet() {
        let mut decoder = BleMidiDecoder::default();
        assert!(decoded(&mut decoder, &[0x80, 0x81, 0x90, 36]).is_empty());
        assert!(decoded(&mut decoder, &[0x80, 0x81, 100, 127]).is_empty());
        assert_eq!(
            decoded(&mut decoder, &[0x80, 0x82, 0xb0, 2, 3]),
            [vec![0xb0, 2, 3]]
        );
    }
}
