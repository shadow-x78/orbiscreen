// Orbiscreen - fec.rs (GPL-3.0-or-later)
// https://github.com/shadow-x78/orbiscreen

use std::sync::OnceLock;

/// Largest data-shard count whose Cauchy columns stay inside GF(256)
/// when four parity rows occupy field elements 0..3.
pub const MAX_FEC_DATA_SHARDS: usize = 252;
pub const FEC_BLOCK_FLAG: u8 = 0x02;

pub fn parity_count(k: usize) -> usize {
    match k {
        0..=3 => 0,
        4..=16 => 2,
        17..=64 => 3,
        _ => 4,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Protect {
    pub data: Vec<Vec<u8>>,
    pub parity: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shards {
    pub data: Vec<Vec<u8>>,
    pub parity: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FecBlock {
    pub index: u8,
    pub count: u8,
    pub shards: Shards,
}

pub fn shard_count(au_len: usize, chunk: usize) -> usize {
    if au_len == 0 || chunk == 0 {
        return 0;
    }
    let k0 = au_len.div_ceil(chunk);
    if parity_count(k0) == 0 {
        k0
    } else {
        (4 + au_len).div_ceil(chunk)
    }
}

pub fn shard_au_blocks(au: &[u8], chunk: usize) -> Vec<FecBlock> {
    let chunk = chunk.max(1);
    if au.is_empty() {
        return Vec::new();
    }
    if shard_count(au.len(), chunk) <= MAX_FEC_DATA_SHARDS {
        return vec![FecBlock {
            index: 0,
            count: 1,
            shards: shard_au(au, chunk),
        }];
    }
    let inner = chunk.saturating_sub(2).max(1);
    let max_slice = (MAX_FEC_DATA_SHARDS * inner).saturating_sub(4).max(1);
    let mut slices = Vec::new();
    let mut off = 0;
    while off < au.len() {
        let mut n = (au.len() - off).min(max_slice);
        while n > 1 && shard_count(n, inner) > MAX_FEC_DATA_SHARDS {
            n -= 1;
        }
        slices.push(shard_au(&au[off..off + n], inner));
        off += n;
    }
    let count = u8::try_from(slices.len()).unwrap_or(u8::MAX);
    slices
        .into_iter()
        .enumerate()
        .map(|(index, shards)| FecBlock {
            index: index as u8,
            count,
            shards,
        })
        .collect()
}

pub fn block_payload(index: u8, count: u8, body: &[u8]) -> Vec<u8> {
    if count <= 1 {
        return body.to_vec();
    }
    let mut out = Vec::with_capacity(2 + body.len());
    out.push(index);
    out.push(count);
    out.extend_from_slice(body);
    out
}

pub fn shard_au(au: &[u8], chunk: usize) -> Shards {
    let chunk = chunk.max(1);
    if au.is_empty() {
        return Shards {
            data: Vec::new(),
            parity: Vec::new(),
        };
    }
    let k0 = au.len().div_ceil(chunk);
    if parity_count(k0) == 0 {
        return Shards {
            data: au.chunks(chunk).map(|c| c.to_vec()).collect(),
            parity: Vec::new(),
        };
    }
    let mut blob = Vec::with_capacity(4 + au.len());
    blob.extend_from_slice(&(au.len() as u32).to_le_bytes());
    blob.extend_from_slice(au);
    let parts: Vec<Vec<u8>> = blob.chunks(chunk).map(|c| c.to_vec()).collect();
    let prot = protect(&parts);
    Shards {
        data: prot.data,
        parity: prot.parity,
    }
}

pub fn protect(parts: &[Vec<u8>]) -> Protect {
    let k = parts.len();
    let m = parity_count(k);
    let width = parts.iter().map(|p| p.len()).max().unwrap_or(0);
    let mut data: Vec<Vec<u8>> = parts
        .iter()
        .map(|p| {
            let mut v = p.clone();
            v.resize(width, 0);
            v
        })
        .collect();
    let parity = if m == 0 || width == 0 || k > MAX_FEC_DATA_SHARDS {
        Vec::new()
    } else {
        encode(&data, m).unwrap_or_default()
    };

    for (i, p) in parts.iter().enumerate() {
        data[i].truncate(p.len());
    }
    Protect { data, parity }
}

pub fn recover(data: &mut [Option<Vec<u8>>], parity: &[Option<Vec<u8>>]) -> bool {
    let k = data.len();
    if k == 0 {
        return true;
    }
    if data.iter().all(|d| d.is_some()) {
        return true;
    }
    let m = parity.len();
    if m == 0 || parity_count(k) == 0 || k > MAX_FEC_DATA_SHARDS {
        return false;
    }
    let width = data
        .iter()
        .chain(parity.iter())
        .filter_map(|s| s.as_ref().map(|v| v.len()))
        .max()
        .unwrap_or(0);
    if width == 0 {
        return false;
    }
    let missing: Vec<usize> = data
        .iter()
        .enumerate()
        .filter_map(|(i, d)| d.is_none().then_some(i))
        .collect();
    let present_p: Vec<usize> = parity
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.as_ref().and_then(|v| (v.len() == width).then_some(i)))
        .collect();
    if missing.len() > present_p.len() {
        return false;
    }
    let used_p = &present_p[..missing.len()];

    let mut known: Vec<Vec<u8>> = vec![Vec::new(); k];
    for (i, slot) in data.iter().enumerate() {
        if let Some(v) = slot {
            let mut p = v.clone();
            p.resize(width, 0);
            known[i] = p;
        }
    }

    let miss_n = missing.len();
    let mut rhs = vec![vec![0u8; width]; miss_n];
    for (row, &p) in used_p.iter().enumerate() {
        let Some(rec_slot) = parity.get(p).and_then(|opt| opt.as_ref()) else {
            return false;
        };
        let mut rec = rec_slot.clone();
        rec.resize(width, 0);
        for byte in 0..width {
            let mut s = rec[byte];
            for d in 0..k {
                if data[d].is_some() {
                    let Some(coeff) = cauchy(p, d, m) else {
                        return false;
                    };
                    s ^= gf_mul(coeff, known[d][byte]);
                }
            }
            rhs[row][byte] = s;
        }
    }

    let mut a = vec![vec![0u8; miss_n]; miss_n];
    for (row, &p) in used_p.iter().enumerate() {
        for (col, &d) in missing.iter().enumerate() {
            let Some(coeff) = cauchy(p, d, m) else {
                return false;
            };
            a[row][col] = coeff;
        }
    }
    let Some(inv) = invert(&a) else {
        return false;
    };

    for (col, &d) in missing.iter().enumerate() {
        let mut out = vec![0u8; width];
        for byte in 0..width {
            let mut s = 0u8;
            for row in 0..miss_n {
                s ^= gf_mul(inv[col][row], rhs[row][byte]);
            }
            out[byte] = s;
        }

        data[d] = Some(out);
    }
    true
}

pub fn concat_fec_au(parts: &[Vec<u8>]) -> Option<Vec<u8>> {
    if parts.is_empty() {
        return None;
    }
    let mut blob = Vec::new();
    for p in parts {
        blob.extend_from_slice(p);
    }
    if blob.len() < 4 {
        return None;
    }
    let n = u32::from_le_bytes(blob[0..4].try_into().ok()?) as usize;
    if n > blob.len().saturating_sub(4) {
        return None;
    }
    Some(blob[4..4 + n].to_vec())
}

#[allow(clippy::needless_range_loop)]
fn encode(data: &[Vec<u8>], m: usize) -> Option<Vec<Vec<u8>>> {
    let k = data.len();
    let width = data[0].len();
    let mut parity = vec![vec![0u8; width]; m];
    for p in 0..m {
        for byte in 0..width {
            let mut s = 0u8;
            for d in 0..k {
                s ^= gf_mul(cauchy(p, d, m)?, data[d][byte]);
            }
            parity[p][byte] = s;
        }
    }
    Some(parity)
}

fn cauchy(p: usize, d: usize, m: usize) -> Option<u8> {
    let y = m.checked_add(d)?;
    if y > u8::MAX as usize {
        return None;
    }
    let denom = (p as u8) ^ (y as u8);
    if denom == 0 {
        return None;
    }
    Some(gf_inv(denom))
}

struct Gf {
    exp: [u8; 512],
    log: [u8; 256],
}

fn gf() -> &'static Gf {
    static T: OnceLock<Gf> = OnceLock::new();
    T.get_or_init(|| {
        let mut exp = [0u8; 512];
        let mut log = [0u8; 256];
        let mut x: u16 = 1;
        #[allow(clippy::needless_range_loop)]
        for i in 0..255 {
            exp[i] = x as u8;
            log[x as usize] = i as u8;
            x <<= 1;
            if x & 0x100 != 0 {
                x ^= 0x11d;
            }
        }
        for i in 255..512 {
            exp[i] = exp[i - 255];
        }
        Gf { exp, log }
    })
}

fn gf_mul(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 {
        return 0;
    }
    let t = gf();
    t.exp[t.log[a as usize] as usize + t.log[b as usize] as usize]
}

fn gf_inv(a: u8) -> u8 {
    let t = gf();
    t.exp[255 - t.log[a as usize] as usize]
}

#[allow(clippy::needless_range_loop)]
fn invert(a: &[Vec<u8>]) -> Option<Vec<Vec<u8>>> {
    let n = a.len();
    if n == 0 {
        return Some(Vec::new());
    }
    let mut m = vec![vec![0u8; 2 * n]; n];
    for i in 0..n {
        for j in 0..n {
            m[i][j] = a[i][j];
        }
        m[i][n + i] = 1;
    }
    for col in 0..n {
        let mut piv = col;
        while piv < n && m[piv][col] == 0 {
            piv += 1;
        }
        if piv == n {
            return None;
        }
        m.swap(col, piv);
        let inv = gf_inv(m[col][col]);
        for j in 0..2 * n {
            m[col][j] = gf_mul(m[col][j], inv);
        }
        for row in 0..n {
            if row == col {
                continue;
            }
            let f = m[row][col];
            if f == 0 {
                continue;
            }
            for j in 0..2 * n {
                m[row][j] ^= gf_mul(f, m[col][j]);
            }
        }
    }
    Some((0..n).map(|i| m[i][n..].to_vec()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladder_skips_tiny_blocks() {
        assert_eq!(parity_count(0), 0);
        assert_eq!(parity_count(1), 0);
        assert_eq!(parity_count(3), 0);
        assert_eq!(parity_count(4), 2);
        assert_eq!(parity_count(16), 2);
        assert_eq!(parity_count(17), 3);
        assert_eq!(parity_count(64), 3);
        assert_eq!(parity_count(65), 4);
        assert_eq!(parity_count(200), 4);
    }

    fn parts(k: usize, width: usize, seed: u8) -> Vec<Vec<u8>> {
        (0..k)
            .map(|i| {
                (0..width)
                    .map(|b| {
                        seed.wrapping_add(i as u8)
                            .wrapping_mul(31)
                            .wrapping_add(b as u8)
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn tiny_block_emits_no_parity() {
        let p = protect(&parts(3, 16, 7));
        assert!(p.parity.is_empty());
        assert_eq!(p.data.len(), 3);
    }

    #[test]
    fn recovers_two_erasures_on_a_four_shard_block() {
        let src = parts(4, 32, 9);
        let prot = protect(&src);
        assert_eq!(prot.parity.len(), 2);
        let mut data: Vec<Option<Vec<u8>>> = prot.data.into_iter().map(Some).collect();
        let parity: Vec<Option<Vec<u8>>> = prot.parity.into_iter().map(Some).collect();
        data[1] = None;
        data[3] = None;
        assert!(recover(&mut data, &parity));
        for i in 0..4 {
            assert_eq!(data[i].as_ref().unwrap(), &src[i]);
        }
    }

    #[test]
    fn recovers_three_erasures_on_a_twenty_shard_block() {
        let src = parts(20, 24, 3);
        let prot = protect(&src);
        assert_eq!(prot.parity.len(), 3);
        let mut data: Vec<Option<Vec<u8>>> = prot.data.into_iter().map(Some).collect();
        let parity: Vec<Option<Vec<u8>>> = prot.parity.into_iter().map(Some).collect();
        data[0] = None;
        data[7] = None;
        data[19] = None;
        assert!(recover(&mut data, &parity));
        for i in 0..20 {
            assert_eq!(data[i].as_ref().unwrap(), &src[i]);
        }
    }

    #[test]
    fn fails_when_erasures_exceed_parity() {
        let src = parts(4, 8, 1);
        let prot = protect(&src);
        let mut data: Vec<Option<Vec<u8>>> = prot.data.into_iter().map(Some).collect();
        let parity: Vec<Option<Vec<u8>>> = prot.parity.into_iter().map(Some).collect();
        data[0] = None;
        data[1] = None;
        data[2] = None;
        assert!(!recover(&mut data, &parity));
    }

    #[test]
    fn shard_au_skips_fec_under_four_fragments() {
        let au = vec![9u8; 30];
        let s = shard_au(&au, 16);
        assert_eq!(s.data.len(), 2);
        assert!(s.parity.is_empty());
        let mut got = Vec::new();
        for p in &s.data {
            got.extend_from_slice(p);
        }
        assert_eq!(got, au);
    }

    #[test]
    fn shard_au_recovers_after_dropping_two_data_shards() {
        let au: Vec<u8> = (0..200u8).collect();
        let s = shard_au(&au, 40);
        assert!(s.data.len() >= 4);
        assert_eq!(s.parity.len(), parity_count(s.data.len()));
        let mut data: Vec<Option<Vec<u8>>> = s.data.into_iter().map(Some).collect();
        let parity: Vec<Option<Vec<u8>>> = s.parity.into_iter().map(Some).collect();
        data[0] = None;
        data[2] = None;
        assert!(recover(&mut data, &parity));
        let parts: Vec<Vec<u8>> = data.into_iter().map(|p| p.unwrap()).collect();
        assert_eq!(concat_fec_au(&parts).unwrap(), au);
    }

    #[test]
    fn two_hundred_fifty_two_data_shards_recover_two_erasures() {
        let src = parts(252, 4, 11);
        let prot = protect(&src);
        assert_eq!(prot.data.len(), 252);
        assert_eq!(prot.parity.len(), 4);
        let mut data: Vec<Option<Vec<u8>>> = prot.data.into_iter().map(Some).collect();
        let parity: Vec<Option<Vec<u8>>> = prot.parity.into_iter().map(Some).collect();
        data[1] = None;
        data[250] = None;
        assert!(recover(&mut data, &parity));
        for (i, src_i) in src.iter().enumerate() {
            assert_eq!(data[i].as_ref().unwrap(), src_i);
        }
    }

    #[test]
    fn more_than_252_data_shards_split_and_still_recover() {
        let chunk = 8usize;
        let au = vec![7u8; 2013];
        let blocks = shard_au_blocks(&au, chunk);
        assert!(blocks.len() > 1);
        assert!(blocks
            .iter()
            .all(|block| block.shards.data.len() <= MAX_FEC_DATA_SHARDS));
        assert!(blocks
            .iter()
            .all(|block| block.count as usize == blocks.len()));

        let mut pieces = Vec::new();
        for block in &blocks {
            let mut data: Vec<Option<Vec<u8>>> =
                block.shards.data.iter().cloned().map(Some).collect();
            let parity: Vec<Option<Vec<u8>>> =
                block.shards.parity.iter().cloned().map(Some).collect();
            if block.index == 0 && !parity.is_empty() {
                data[0] = None;
                assert!(recover(&mut data, &parity));
            }
            let parts: Vec<Vec<u8>> = data.into_iter().map(|part| part.unwrap()).collect();
            let slice = if block.shards.parity.is_empty() {
                parts.into_iter().flatten().collect()
            } else {
                concat_fec_au(&parts).unwrap()
            };
            pieces.extend(slice);
        }
        assert_eq!(pieces, au);
    }

    #[test]
    fn concat_strips_length_prefix() {
        let au = vec![0, 0, 0, 1, 0x41, 9, 8, 7];
        let mut blob = (au.len() as u32).to_le_bytes().to_vec();
        blob.extend_from_slice(&au);
        let chunk = 5;
        let parts: Vec<Vec<u8>> = blob.chunks(chunk).map(|c| c.to_vec()).collect();
        assert_eq!(concat_fec_au(&parts).unwrap(), au);
    }
}
