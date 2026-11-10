#[derive(Default)]
pub struct MidiStream {
    status: u8,
    remaining: usize,
    position: usize,
    data: [u8; 3],
    sysex: bool,
}

impl MidiStream {
    pub fn feed(&mut self, bytes: &[u8], mut message: impl FnMut(&[u8])) {
        for &byte in bytes {
            match byte {
                0xf8..=0xff => continue,
                0xf0..=0xf7 => {
                    self.status = 0;
                    self.sysex = byte == 0xf0;
                }
                0x80..=0xef => {
                    self.sysex = false;
                    self.status = byte;
                    self.remaining = if matches!(byte & 0xf0, 0xc0 | 0xd0) {
                        1
                    } else {
                        2
                    };
                    self.position = 0;
                    self.data[0] = byte;
                }
                _ if !self.sysex && self.status != 0 => {
                    self.position += 1;
                    self.data[self.position] = byte;
                    if self.position == self.remaining {
                        message(&self.data[..=self.position]);
                        self.position = 0;
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MidiStream;

    #[test]
    fn frames_fragmented_and_running_status_messages_with_interleaved_clock() {
        let mut midi = MidiStream::default();
        let mut messages = Vec::new();
        let mut collect = |data: &[u8]| messages.push(data.to_vec());
        midi.feed(&[0x91, 42, 0xf8], &mut collect);
        midi.feed(&[100, 43, 90, 0xc1, 2, 3], &mut collect);
        assert_eq!(
            messages,
            [
                vec![0x91, 42, 100],
                vec![0x91, 43, 90],
                vec![0xc1, 2],
                vec![0xc1, 3]
            ]
        );
    }

    #[test]
    fn system_messages_clear_running_status() {
        let mut midi = MidiStream::default();
        let mut messages = Vec::new();
        midi.feed(
            &[0x90, 42, 0xf0, 9, 0xf7, 100, 0xf2, 1, 2, 0x90, 11, 12],
            |data| messages.push(data.to_vec()),
        );
        assert_eq!(messages, [vec![0x90, 11, 12]]);
    }
}
