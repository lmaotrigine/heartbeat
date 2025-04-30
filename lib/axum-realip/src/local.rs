use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, Copy)]
struct IpNet {
    addr: IpAddr,
    mask: u8,
}

pub enum UnsignedInteger {
    V4(u32),
    V6(u128),
}

impl UnsignedInteger {
    const fn u32(self) -> u32 {
        match self {
            Self::V4(v) => v,
            Self::V6(_) => panic!("tried to cast IPv6 to u32"),
        }
    }

    const fn u128(self) -> u128 {
        match self {
            Self::V6(v) => v,
            Self::V4(_) => panic!("tried to cast IPv4 to u128"),
        }
    }
}

impl IpNet {
    const fn new(addr: IpAddr, mask: u8) -> Self {
        Self { addr, mask }
    }

    const fn native_host_mask(&self) -> UnsignedInteger {
        match self.addr {
            IpAddr::V4(_) => UnsignedInteger::V4(if let Some(mask) = (!0u32).checked_shr(self.mask as u32) {
                mask
            } else {
                0
            }),
            IpAddr::V6(_) => UnsignedInteger::V6(if let Some(mask) = (!0u128).checked_shr(self.mask as u32) {
                mask
            } else {
                0
            }),
        }
    }

    const fn to_native(addr: IpAddr) -> UnsignedInteger {
        match addr {
            IpAddr::V4(ip) => UnsignedInteger::V4(u32::from_be_bytes(ip.octets())),
            IpAddr::V6(ip) => UnsignedInteger::V6(u128::from_be_bytes(ip.octets())),
        }
    }

    const fn prefix_match(&self, other: IpAddr) -> bool {
        if !assert_same_type(self.addr, other) {
            return false;
        }
        match self.native_host_mask() {
            UnsignedInteger::V4(mask) => {
                (Self::to_native(self.addr).u32() & mask) == (Self::to_native(other).u32() & mask)
            }
            UnsignedInteger::V6(mask) => {
                (Self::to_native(self.addr).u128() & mask) == (Self::to_native(other).u128() & mask)
            }
        }
    }

    pub const fn contains(&self, addr: &IpAddr) -> bool {
        self.prefix_match(*addr)
    }
}

const fn assert_same_type(a: IpAddr, b: IpAddr) -> bool {
    matches!((a, b), (IpAddr::V4(_), IpAddr::V4(_)) | (IpAddr::V6(_), IpAddr::V6(_)))
}

static IPV4_LOCAL_SUBNETS: &[IpNet] = &[
    IpNet::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8), // localhost
    IpNet::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)), 8),  // 24-bit block
    IpNet::new(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 0)), 12), // 20-bit block
    IpNet::new(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0)), 16), // 16-bit block
    IpNet::new(IpAddr::V4(Ipv4Addr::new(169, 254, 0, 0)), 16), // link local
];

static IPV6_LOCAL_SUBNETS: &[IpNet] = &[
    IpNet::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 128), // localhost
    IpNet::new(IpAddr::V6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0)), 7), // unique local
    IpNet::new(IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0)), 10), // link local
];

pub trait IsLocalAddr {
    fn is_local(&self) -> bool;
}

impl IsLocalAddr for Ipv4Addr {
    fn is_local(&self) -> bool {
        IPV4_LOCAL_SUBNETS
            .iter()
            .any(|subnet| subnet.contains(&IpAddr::V4(*self)))
    }
}

impl IsLocalAddr for Ipv6Addr {
    fn is_local(&self) -> bool {
        IPV6_LOCAL_SUBNETS
            .iter()
            .any(|subnet| subnet.contains(&IpAddr::V6(*self)))
    }
}

impl IsLocalAddr for IpAddr {
    fn is_local(&self) -> bool {
        match self {
            Self::V4(ip) => ip.is_local(),
            Self::V6(ip) => ip.is_local(),
        }
    }
}
