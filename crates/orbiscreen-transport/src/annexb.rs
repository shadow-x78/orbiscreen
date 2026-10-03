#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpsPps {
    pub sps: Vec<u8>,
    pub pps: Vec<u8>,
}

pub fn start_code_len(data: &[u8], i: usize) -> Option<usize> {
    if i + 3 < data.len() && data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1 {
        return Some(3);
    }
    if i + 4 < data.len()
        && data[i] == 0
        && data[i + 1] == 0
        && data[i + 2] == 0
        && data[i + 3] == 1
    {
        return Some(4);
    }
    None
}

pub fn nal_type(nal: &[u8]) -> Option<u8> {
    nal.first().map(|b| b & 0x1f)
}

pub fn extract_sps_pps(au: &[u8]) -> SpsPps {
    let mut sps = Vec::new();
    let mut pps = Vec::new();
    let mut i = 0;
    while i < au.len() {
        let Some(sc) = start_code_len(au, i) else {
            break;
        };
        let nal_start = i + sc;
        let mut next = nal_start;
        while next < au.len() {
            if start_code_len(au, next).is_some() && next > nal_start {
                break;
            }
            next += 1;
        }
        if nal_start < next {
            let nal = &au[nal_start..next];
            match nal_type(nal) {
                Some(7) => sps = with_start_code(nal),
                Some(8) => pps = with_start_code(nal),
                _ => {}
            }
        }
        i = next;
    }
    SpsPps { sps, pps }
}

pub fn with_start_code(nal: &[u8]) -> Vec<u8> {
    let mut out = vec![0, 0, 0, 1];
    out.extend_from_slice(nal);
    out
}

pub fn codec_string_from_sps(sps_annexb: &[u8]) -> Option<String> {
    let sc = start_code_len(sps_annexb, 0)?;
    let nal = sps_annexb.get(sc..)?;
    if nal_type(nal) != Some(7) || nal.len() < 4 {
        return None;
    }
    Some(format!("avc1.{:02X}{:02X}{:02X}", nal[1], nal[2], nal[3]))
}

pub fn is_annexb(bytes: &[u8]) -> bool {
    start_code_len(bytes, 0).is_some()
}

pub fn with_parameter_sets(au: &[u8], cached: Option<&SpsPps>) -> Vec<u8> {
    let found = extract_sps_pps(au);
    if !found.sps.is_empty() && !found.pps.is_empty() {
        return au.to_vec();
    }
    let Some(cached) = cached else {
        return au.to_vec();
    };
    let sps = if found.sps.is_empty() {
        cached.sps.as_slice()
    } else {
        found.sps.as_slice()
    };
    let pps = if found.pps.is_empty() {
        cached.pps.as_slice()
    } else {
        found.pps.as_slice()
    };
    if sps.is_empty() || pps.is_empty() {
        return au.to_vec();
    }
    let mut out = Vec::with_capacity(sps.len() + pps.len() + au.len());
    out.extend_from_slice(sps);
    out.extend_from_slice(pps);
    out.extend_from_slice(au);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_sps() -> Vec<u8> {
        let mut nal = vec![0x67, 0x64, 0x00, 0x28];
        nal.extend_from_slice(&[0xac, 0x2b]);
        with_start_code(&nal)
    }

    fn sample_pps() -> Vec<u8> {
        with_start_code(&[0x68, 0xee, 0x3c, 0x80])
    }

    #[test]
    fn start_code_3_and_4() {
        assert_eq!(start_code_len(&[0, 0, 1, 0x65], 0), Some(3));
        assert_eq!(start_code_len(&[0, 0, 0, 1, 0x65], 0), Some(4));
        assert_eq!(start_code_len(&[0, 1, 0, 1], 0), None);
    }

    #[test]
    fn extracts_sps_pps_from_idr_au() {
        let mut au = sample_sps();
        au.extend_from_slice(&sample_pps());
        au.extend_from_slice(&with_start_code(&[0x65, 0x88, 0x84]));
        let found = extract_sps_pps(&au);
        assert_eq!(found.sps, sample_sps());
        assert_eq!(found.pps, sample_pps());
        assert_eq!(
            codec_string_from_sps(&found.sps).as_deref(),
            Some("avc1.640028")
        );
    }

    #[test]
    fn codec_string_rejects_non_sps() {
        assert_eq!(codec_string_from_sps(&sample_pps()), None);
        assert_eq!(codec_string_from_sps(&[0, 0, 0, 1, 0x67]), None);
    }

    #[test]
    fn annexb_probe() {
        assert!(is_annexb(&[0, 0, 0, 1, 0x65]));
        assert!(!is_annexb(&[0, 1, 0, 1]));
    }

    #[test]
    fn with_parameter_sets_leaves_complete_idr_alone() {
        let mut au = sample_sps();
        au.extend_from_slice(&sample_pps());
        au.extend_from_slice(&with_start_code(&[0x65, 0x88]));
        assert_eq!(with_parameter_sets(&au, None), au);
    }

    #[test]
    fn with_parameter_sets_prepends_cached_when_idr_lacks_headers() {
        let slice = with_start_code(&[0x65, 0x88, 0x84]);
        let cached = SpsPps {
            sps: sample_sps(),
            pps: sample_pps(),
        };
        let out = with_parameter_sets(&slice, Some(&cached));
        let found = extract_sps_pps(&out);
        assert_eq!(found.sps, sample_sps());
        assert_eq!(found.pps, sample_pps());
        assert!(out.ends_with(&slice));
    }
}
