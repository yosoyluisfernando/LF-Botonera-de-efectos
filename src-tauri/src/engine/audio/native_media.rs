//! Lectura de AAC ADTS MPEG-2 con el lector ADTS MPEG-4 de Symphonia 0.5.
//! Solo adapta el bit ID en cabeceras indexadas; nunca modifica muestras ni archivos.
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use symphonia::core::io::MediaSource;

pub fn open(path: &str) -> io::Result<Box<dyn MediaSource>> {
    let mut file = File::open(path)?;
    let mut prefix = [0; 2];
    let count = file.read(&mut prefix)?;
    file.seek(SeekFrom::Start(0))?;
    if count != 2 || prefix != [0xff, 0xf9] {
        return Ok(Box::new(file));
    }
    let size = file.metadata()?.len();
    let mut offset = 0u64;
    let mut ids = Vec::new();
    while offset < size {
        file.seek(SeekFrom::Start(offset))?;
        let mut header = [0; 7];
        file.read_exact(&mut header)?;
        if header[0] != 0xff || ![0xf9, 0xf1].contains(&header[1]) || header[2] >> 6 != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported ADTS header",
            ));
        }
        let len =
            ((header[3] as u64 & 3) << 11) | ((header[4] as u64) << 3) | (header[5] as u64 >> 5);
        if len < 7 || offset + len > size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid ADTS frame length",
            ));
        }
        if header[1] == 0xf9 {
            ids.push(offset + 1);
        }
        offset += len;
    }
    file.seek(SeekFrom::Start(0))?;
    Ok(Box::new(AdtsFile {
        file,
        ids,
        position: 0,
        size,
    }))
}
struct AdtsFile {
    file: File,
    ids: Vec<u64>,
    position: u64,
    size: u64,
}
impl Read for AdtsFile {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let count = self.file.read(bytes)?;
        let end = self.position + count as u64;
        let first = self.ids.partition_point(|offset| *offset < self.position);
        for offset in self.ids[first..].iter().take_while(|offset| **offset < end) {
            bytes[(offset - self.position) as usize] &= !0x08;
        }
        self.position = end;
        Ok(count)
    }
}
impl Seek for AdtsFile {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.position = self.file.seek(position)?;
        Ok(self.position)
    }
}
impl MediaSource for AdtsFile {
    fn is_seekable(&self) -> bool {
        true
    }
    fn byte_len(&self) -> Option<u64> {
        Some(self.size)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn header_adaptation_preserves_payload_original_and_random_seek() {
        let path = std::env::temp_dir().join(format!("lf-adts-native-{}.aac", std::process::id()));
        let frame = [
            0xff, 0xf9, 0x50, 0x80, 0x02, 0x1f, 0xfc, 0, 0, 0xff, 0xf9, 0, 0, 0, 0, 0,
        ];
        let original = [frame, frame].concat();
        std::fs::write(&path, &original).unwrap();
        let mut source = open(path.to_str().unwrap()).unwrap();
        let mut read = Vec::new();
        source.read_to_end(&mut read).unwrap();
        let mut expected = original.clone();
        expected[1] = 0xf1;
        expected[17] = 0xf1;
        assert_eq!(read, expected);
        source.seek(SeekFrom::Start(9)).unwrap();
        let mut payload = [0; 2];
        source.read_exact(&mut payload).unwrap();
        assert_eq!(payload, [0xff, 0xf9]);
        source.seek(SeekFrom::Start(1)).unwrap();
        let mut id = [0];
        source.read_exact(&mut id).unwrap();
        assert_eq!(id, [0xf1]);
        assert_eq!(std::fs::read(&path).unwrap(), original);
        drop(source);
        std::fs::remove_file(path).unwrap();
    }
}
