pub const DEFAULT_SIGNALING_PORT: u16 = 8788;
pub const UDP_OFFSET: u16 = 1;
pub const HTTPS_OFFSET: u16 = 2;

pub fn udp_port(signaling_port: u16) -> u16 {
    signaling_port.saturating_add(UDP_OFFSET)
}

pub fn https_port(signaling_port: u16) -> u16 {
    signaling_port.saturating_add(HTTPS_OFFSET)
}

pub fn wt_port(signaling_port: u16) -> u16 {
    https_port(signaling_port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ports_are_contiguous() {
        assert_eq!(DEFAULT_SIGNALING_PORT, 8788);
        assert_eq!(udp_port(8788), 8789);
        assert_eq!(https_port(8788), 8790);
        assert_eq!(wt_port(8788), 8790);
    }

    #[test]
    fn port_arithmetic_saturates_at_the_top() {
        assert_eq!(udp_port(u16::MAX), u16::MAX);
        assert_eq!(https_port(u16::MAX), u16::MAX);
    }
}
