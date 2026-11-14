use crate::frame_pool::PooledFrameBuffer;

pub const BYTES_PER_PIXEL: usize = 4;

pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub data: PooledFrameBuffer,
}

impl std::fmt::Debug for CapturedFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapturedFrame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("data_len", &self.data.len())
            .finish()
    }
}

impl CapturedFrame {
    pub fn size_in_bytes(width: u32, height: u32) -> usize {
        frame_len(width, height).unwrap_or(usize::MAX)
    }
}

pub fn frame_len(width: u32, height: u32) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(BYTES_PER_PIXEL))
}

pub fn assemble_bgra_rows(
    dst: &mut [u8],
    src: &[u8],
    width: u32,
    height: u32,
    stride: u32,
    swap_rb: bool,
    opaque_alpha: bool,
) -> bool {
    let Some(row_bytes) = (width as usize).checked_mul(BYTES_PER_PIXEL) else {
        return false;
    };
    let Some(expected) = row_bytes.checked_mul(height as usize) else {
        return false;
    };
    if dst.len() != expected {
        tracing::warn!("frame destination size mismatch; dropping frame");
        return false;
    }
    let stride = stride as usize;
    if stride < row_bytes && height > 0 {
        tracing::warn!("frame stride is shorter than a row; dropping frame");
        return false;
    }
    for row in 0..height as usize {
        let Some(start) = row.checked_mul(stride) else {
            return false;
        };
        let Some(end) = start.checked_add(row_bytes) else {
            return false;
        };
        if end > src.len() {
            tracing::warn!("frame buffer shorter than expected; dropping frame");
            return false;
        }
        let dst = &mut dst[row * row_bytes..row * row_bytes + row_bytes];
        dst.copy_from_slice(&src[start..end]);
        if swap_rb || opaque_alpha {
            for px in dst.as_chunks_mut::<BYTES_PER_PIXEL>().0 {
                if swap_rb {
                    px.swap(0, 2);
                }
                if opaque_alpha {
                    px[3] = 0xFF;
                }
            }
        }
    }
    true
}

pub fn bgra_from_rgb565(src: &[u8], src_stride: u32, width: u32, height: u32) -> Option<Vec<u8>> {
    let capacity = frame_len(width, height)?;
    let mut data = Vec::with_capacity(capacity);
    for row in 0..height as usize {
        let start = row.checked_mul(src_stride as usize)?;
        for col in 0..width as usize {
            let off = start.checked_add(col.checked_mul(2)?)?;
            let bytes = src.get(off..off + 2)?;
            let pixel = u16::from_le_bytes([bytes[0], bytes[1]]);
            let r = ((pixel >> 11) & 0x1F) as u8;
            let g = ((pixel >> 5) & 0x3F) as u8;
            let b = (pixel & 0x1F) as u8;
            let r8 = (r << 3) | (r >> 2);
            let g8 = (g << 2) | (g >> 4);
            let b8 = (b << 3) | (b >> 2);
            data.extend_from_slice(&[b8, g8, r8, 0xFF]);
        }
    }
    Some(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assemble(
        src: &[u8],
        width: u32,
        height: u32,
        stride: u32,
        opaque_alpha: bool,
        swap_rb: bool,
    ) -> Option<Vec<u8>> {
        let mut dst = vec![0u8; frame_len(width, height)?];
        assemble_bgra_rows(&mut dst, src, width, height, stride, swap_rb, opaque_alpha)
            .then_some(dst)
    }

    #[test]
    fn size_in_bytes_is_w_h_4() {
        assert_eq!(CapturedFrame::size_in_bytes(1920, 1080), 1920 * 1080 * 4);
        assert_eq!(CapturedFrame::size_in_bytes(0, 0), 0);
    }

    #[test]
    fn assembly_swaps_red_and_blue_for_xbgr_sources() {
        let src = [0x11, 0x22, 0x33, 0x44];
        let data = assemble(&src, 1, 1, 4, false, true).expect("assembly");
        assert_eq!(data, [0x33, 0x22, 0x11, 0x44]);
    }

    #[test]
    fn assembly_swaps_and_fills_alpha_for_xbgr_sources() {
        let src = [0xAA, 0xBB, 0xCC, 0x00];
        let data = assemble(&src, 1, 1, 4, true, true).expect("assembly");
        assert_eq!(data, [0xCC, 0xBB, 0xAA, 0xFF]);
    }

    #[test]
    fn assembly_leaves_bgra_sources_untouched_without_flags() {
        let src = [0x11, 0x22, 0x33, 0xFF, 0x44, 0x55, 0x66, 0xFF];
        let data = assemble(&src, 2, 1, 8, false, false).expect("assembly");
        assert_eq!(data, src);
    }

    #[test]
    fn assembly_copies_contiguous_rows() {
        let src = vec![7u8; 4 * 3];
        let data = assemble(&src, 3, 1, 12, false, false).expect("assembly");
        assert_eq!(data, src);
    }

    #[test]
    fn assembly_strips_stride_padding() {
        let src = [
            0xAA, 0xBB, 0xCC, 0xDD, 0x01, 0x02, 0x03, 0x04, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0x11, 0x22, 0x33, 0x44, 0x05, 0x06, 0x07, 0x08,
        ];
        let data = assemble(&src, 2, 2, 16, false, false).expect("assembly");
        assert_eq!(data.len(), 16);
        assert_eq!(&data[..8], &src[..8]);
        assert_eq!(data[8..], src[16..24]);
    }

    #[test]
    fn assembly_forces_opaque_alpha_for_premultiplied_xrgb() {
        let src = [0x10, 0x20, 0x30, 0x00, 0x40, 0x50, 0x60, 0x7F];
        let data = assemble(&src, 2, 1, 8, true, false).expect("assembly");
        assert_eq!(data[3], 0xFF);
        assert_eq!(data[7], 0xFF);
        assert_eq!(&data[..3], &src[..3]);
        assert_eq!(&data[4..7], &src[4..7]);
    }

    #[test]
    fn assembly_keeps_alpha_for_argb_source() {
        let src = [0x10, 0x20, 0x30, 0x7F];
        let data = assemble(&src, 1, 1, 4, false, false).expect("assembly");
        assert_eq!(data[3], 0x7F);
    }

    #[test]
    fn assembly_rejects_short_buffers() {
        let src = vec![0u8; 7];
        assert!(assemble(&src, 2, 1, 8, false, false).is_none());
        assert!(assemble(&src, 1, 2, 4, false, false).is_none());
    }

    #[test]
    fn assembly_handles_zero_sized_frames() {
        let data = assemble(&[], 0, 0, 0, true, false).expect("assembly");
        assert!(data.is_empty());
    }

    #[test]
    fn rgb565_is_expanded_to_opaque_bgra() {
        let bytes = [0xFF, 0xFF];
        let data = bgra_from_rgb565(&bytes, 2, 1, 1).expect("convert");
        assert_eq!(data, vec![0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn rgb565_rejects_truncated_rows() {
        let bytes = [0xFF];
        assert!(bgra_from_rgb565(&bytes, 2, 1, 1).is_none());
    }
}
