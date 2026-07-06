/// Lectura/escritura binaria versionada de WaveEnvelope.
use crate::engine::dsp::waveform::WaveEnvelope;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

const MAGIC: &[u8; 4] = b"LFWF";
const VERSION: u32 = 1;

pub fn read(path: &PathBuf) -> Result<WaveEnvelope, String> {
    let mut f = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut magic = [0u8; 4];
    f.read_exact(&mut magic).map_err(|e| e.to_string())?;
    if &magic != MAGIC || read_u32(&mut f)? != VERSION {
        return Err("invalid_waveform_cache".to_string());
    }
    let sample_rate = read_u32(&mut f)?;
    let frames = read_u64(&mut f)? as usize;
    let points = read_u32(&mut f)? as usize;
    let mut mins = Vec::with_capacity(points);
    let mut maxs = Vec::with_capacity(points);
    for _ in 0..points {
        mins.push(read_f32(&mut f)?);
        maxs.push(read_f32(&mut f)?);
    }
    Ok(WaveEnvelope::from_parts(mins, maxs, sample_rate, frames))
}

pub fn write(path: &PathBuf, env: &WaveEnvelope) -> Result<u64, String> {
    let (mins, maxs, sample_rate, frames) = env.parts();
    let mut f = fs::File::create(path).map_err(|e| e.to_string())?;
    f.write_all(MAGIC).map_err(|e| e.to_string())?;
    write_u32(&mut f, VERSION)?;
    write_u32(&mut f, sample_rate)?;
    write_u64(&mut f, frames as u64)?;
    write_u32(&mut f, mins.len() as u32)?;
    for i in 0..mins.len() {
        write_f32(&mut f, mins[i])?;
        write_f32(&mut f, maxs[i])?;
    }
    Ok(f.metadata().map_err(|e| e.to_string())?.len())
}

fn read_u32<R: Read>(r: &mut R) -> Result<u32, String> {
    let mut b = [0; 4];
    r.read_exact(&mut b).map_err(|e| e.to_string())?;
    Ok(u32::from_le_bytes(b))
}

fn read_u64<R: Read>(r: &mut R) -> Result<u64, String> {
    let mut b = [0; 8];
    r.read_exact(&mut b).map_err(|e| e.to_string())?;
    Ok(u64::from_le_bytes(b))
}

fn read_f32<R: Read>(r: &mut R) -> Result<f32, String> {
    Ok(f32::from_bits(read_u32(r)?))
}

fn write_u32<W: Write>(w: &mut W, v: u32) -> Result<(), String> {
    w.write_all(&v.to_le_bytes()).map_err(|e| e.to_string())
}

fn write_u64<W: Write>(w: &mut W, v: u64) -> Result<(), String> {
    w.write_all(&v.to_le_bytes()).map_err(|e| e.to_string())
}

fn write_f32<W: Write>(w: &mut W, v: f32) -> Result<(), String> {
    write_u32(w, v.to_bits())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_waveform_roundtrip() {
        let path = std::env::temp_dir().join(format!("lf_wf_{}.wfc", std::process::id()));
        let env = WaveEnvelope::from_parts(vec![-0.5, -0.2], vec![0.4, 0.9], 48_000, 96_000);
        write(&path, &env).unwrap();
        let got = read(&path).unwrap();
        let _ = fs::remove_file(path);
        assert_eq!(got.parts().0, &[-0.5, -0.2]);
        assert_eq!(got.parts().1, &[0.4, 0.9]);
        assert_eq!(got.duration_s(), 2.0);
    }
}
