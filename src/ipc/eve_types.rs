use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ResultData {
    pub key: String,
    pub last_error: String,
    pub last_failed: String,
    #[serde(rename = "LastIPAndDNS")]
    pub last_ip_and_dns: String,
    pub last_succeeded: String,
    pub ports: Vec<Port>,
    pub sha_file: String,
    pub sha_value: Option<String>,
    pub state: u8,
    pub time_priority: String,
    pub version: u8,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Port {
    alias: String,
    cost: u32,
    dhcp_config: DhcpConfig,
    if_name: String,
    is_l3_port: bool,
    is_mgmt: bool,
    l2_link_config: L2LinkConfig,
    #[serde(rename = "Logicallabel")]
    logical_label: String,
    #[serde(rename = "NetworkUUID")]
    network_uuid: Uuid,
    #[serde(rename = "PCIAddr")]
    pci_addr: String,
    #[serde(rename = "Phylabel")]
    phy_label: String,
    proxy_config: ProxyConfig,
    test_results: TestResults,
    #[serde(rename = "USBAddr")]
    usb_addr: String,
    wireless_cfg: WirelessCfg,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Bond {
    #[serde(rename = "ARPMonitor")]
    pub arp_monitor: ArpMonitor,
    pub aggregated_ports: Option<String>,
    pub lacp_rate: LacpRate,
    #[serde(rename = "MIIMonitor")]
    pub mii_monitor: MIIMonitor,
    pub mode: BondMode,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ArpMonitor {
    pub enabled: bool,
    pub ip_targets: Option<String>,
    pub interval: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MIIMonitor {
    pub enabled: bool,
    pub interval: u32,
    pub up_delay: u32,
    pub down_delay: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Vlan {
    pub id: u32,
    pub parent_port: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct WirelessCfg {
    pub cellular: Option<String>,
    pub cellular_v2: CellularV2,
    pub w_type: WirelessType,
    pub wifi: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CellularV2 {
    pub access_points: Option<String>,
    pub location_tracking: bool,
    pub probe: Probe,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Probe {
    pub address: String,
    pub disable: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeviceNetworkStatus {
    #[serde(rename = "DPCKey")]
    pub dpc_key: String,
    pub version: DevicePortConfigVersion,
    pub testing: bool,
    pub state: DPCState,
    pub current_index: i32,
    pub radio_silence: RadioSilence,
    pub ports: Option<Vec<NetworkPortStatus>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct RadioSilence {
    pub imposed: bool,
    pub change_in_progress: bool,
    pub change_requested_at: DateTime<Utc>,
    pub config_error: String,
}

// "subnet": {
//     "IP": "192.168.1.0",
//     "Mask": "////AA=="
// },

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct GoIpNetwork {
    #[serde(rename = "IP")]
    pub ip: String,
    pub mask: String,
}

fn deserialize_ipaddr<'de, D>(deserializer: D) -> Result<Option<IpAddr>, D::Error>
where
    D: Deserializer<'de>,
{
    // if string is empty, return None
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);

    // if string is not empty, parse it as an IP address
    } else {
        let ip = s.parse().map_err(serde::de::Error::custom)?;
        Ok(Some(ip))
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkPortStatus {
    pub if_name: String,
    #[serde(rename = "Phylabel")]
    pub phy_label: String,
    #[serde(rename = "Logicallabel")]
    pub logical_label: String,
    pub alias: String,
    pub is_mgmt: bool,
    pub is_l3_port: bool,
    pub cost: u8,
    pub dhcp: DhcpType,
    #[serde(rename = "Type")]
    pub network_type: NetworkType,
    pub subnet: GoIpNetwork,
    #[serde(deserialize_with = "deserialize_ipaddr")]
    pub ntp_server: Option<IpAddr>,
    pub domain_name: String,
    pub dns_servers: Option<Vec<IpAddr>>,
    pub ntp_servers: Option<Vec<IpAddr>>,
    #[serde(skip)]
    pub addr_info_list: Vec<AddrInfo>,
    pub up: bool,
    pub mac_addr: String, // Alternatively use MacAddress type
    pub default_routers: Option<Vec<IpAddr>>,
    #[serde(rename = "MTU")]
    pub mtu: u16,
    pub wireless_cfg: WirelessConfig,
    pub wireless_status: WirelessStatus,
    #[serde(flatten)]
    pub proxy_config: ProxyConfig,
    #[serde(flatten)]
    pub l2_link_config: L2LinkConfig,
    #[serde(flatten)]
    pub test_results: TestResults,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ProxyConfig {
    proxies: Option<Vec<ProxyEntry>>,
    exceptions: String,
    pacfile: String,
    network_proxy_enable: bool,
    #[serde(rename = "NetworkProxyURL")]
    network_proxy_url: String,
    #[serde(rename = "WpadURL")]
    wpad_url: String,
    proxy_cert_pem: Option<Vec<Vec<u8>>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct L2LinkConfig {
    l2_type: L2LinkType,
    vlan: Option<VLANConfig>,
    bond: Option<BondConfig>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct TestResults {
    // #[serde(with = "ts_seconds")]
    last_failed: DateTime<Utc>,
    // #[serde(with = "ts_seconds")]
    last_succeeded: DateTime<Utc>,
    last_error: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct WirelessStatus {
    w_type: WirelessType,
    #[serde(skip)]
    cellular: WwanNetworkStatus,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ProxyEntry {
    #[serde(rename = "Type")]
    proxy_type: NetworkProxyType,
    server: String,
    port: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddrInfo {
    // Define the fields
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct WifiConfig {
    // Define the fields
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct DeprecatedCellConfig {
    // Define the fields
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct WwanNetworkStatus {
    // Define the fields
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct CellNetPortConfig {
    #[serde(skip)]
    access_points: Vec<CellularAccessPoint>,
    probe: WwanProbe,
    location_tracking: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct WwanProbe {
    disable: bool,
    // IP/FQDN address to periodically probe to determine connection status.
    address: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct CellularAccessPoint {
    // pub key: String, // SIM card slot to which this configuration applies.
    // // 0 - unspecified (apply to currently activated or the only available)
    // // 1 - config for SIM card in the first slot
    // // 2 - config for SIM card in the second slot
    // // etc.
    // pub sim_slot: u8,
    // // If true, then this configuration is currently activated.
    // pub activated: bool,
    // // Access Point Network
    // pub apn: String,
    // // Authentication protocol used by the network.
    // pub auth_protocol: WwanAuthProtocol,
    // // EncryptedCredentials : encrypted username and password.
    // pub encrypted_credentials: CipherBlockStatus,
    // // The set of cellular network operators that modem should preferably try to register
    // // and connect into.
    // // Network operator should be referenced by PLMN (Public Land Mobile Network) code.
    // pub preferred_plmns: Vec<String>,
    // // The list of preferred Radio Access Technologies (RATs) to use for connecting
    // // to the network.
    // pub preferred_rats: Vec<WwanRAT>,
    // // If true, then modem will avoid connecting to networks with roaming.
    // pub forbid_roaming: bool,
}

#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
pub enum L2LinkType {
    // L2LinkTypeNone : not an L2 link (used for physical network adapters).
    L2LinkTypeNone = 0,
    // L2LinkTypeVLAN : VLAN sub-interface
    L2LinkTypeVLAN = 1,
    // L2LinkTypeBond : Bond interface
    L2LinkTypeBond = 2,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct VLANConfig {
    parent_port: String,
    #[serde(rename = "ID")]
    id: u16,
}

#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
pub enum BondMode {
    // BondModeUnspecified : default is Round-Robin
    BondModeUnspecified = 0,
    // BondModeBalanceRR : Round-Robin
    BondModeBalanceRR = 1,
    // BondModeActiveBackup : Active/Backup
    BondModeActiveBackup = 2,
    // BondModeBalanceXOR : select slave for a packet using a hash function
    BondModeBalanceXOR = 3,
    // BondModeBroadcast : send every packet on all slaves
    BondModeBroadcast = 4,
    // BondMode802Dot3AD : IEEE 802.3ad Dynamic link aggregation
    BondMode802Dot3AD = 5,
    // BondModeBalanceTLB : Adaptive transmit load balancing
    BondModeBalanceTLB = 6,
    // BondModeBalanceALB : Adaptive load balancing
    BondModeBalanceALB = 7,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct BondConfig {
    pub aggregated_ports: Option<Vec<String>>,
    pub mode: BondMode,
    pub lacp_rate: LacpRate,
    #[serde(rename = "MIIMonitor")]
    pub mii_monitor: BondMIIMonitor,
    #[serde(rename = "ARPMonitor")]
    pub arp_monitor: BondArpMonitor,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct BondMIIMonitor {
    pub down_delay: u32,
    pub enabled: bool,
    pub interval: u32,
    pub up_delay: u32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct BondArpMonitor {
    pub enabled: bool,
    pub ip_targets: Option<String>,
    pub interval: u32,
}

#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
pub enum LacpRate {
    LacpRateUnspecified = 0,
    LacpRateSlow = 1,
    LacpRateFast = 2,
}

// snippet three
// DhcpType enum
#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
pub enum DhcpType {
    NOOP = 0,
    Static = 1,
    None = 2,
    Deprecated = 3,
    Client = 4,
}

// DPCState enum
#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
pub enum DPCState {
    None = 0,
    Fail = 1,
    FailWithIPAndDNS = 2,
    Success = 3,
    IPDNSWait = 4,
    PCIWait = 5,
    IntfWait = 6,
    RemoteWait = 7,
    AsyncWait = 8,
}

// NetworkType enum
#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
pub enum NetworkType {
    NOOP = 0,
    IPv4 = 4,
    IPV6 = 6,
    Ipv4Only = 5,
    Ipv6Only = 7,
    DualStack = 8,
}

// NetworkProxyType enum
#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
pub enum NetworkProxyType {
    HTTP = 0,
    HTTPS = 1,
    SOCKS = 2,
    FTP = 3,
    NOPROXY = 4,
    LAST = 255,
}

// WirelessType enum
#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
pub enum WirelessType {
    None = 0,
    Cellular = 1,
    Wifi = 2,
}

// WirelessConfig struct
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct WirelessConfig {
    pub w_type: WirelessType,
    pub cellular_v2: CellNetPortConfig,
    #[serde(skip)]
    pub wifi: Option<Vec<WifiConfig>>,
    #[serde(skip)]
    pub cellular: Option<Vec<DeprecatedCellConfig>>,
}

// DevicePortConfigVersion type
pub type DevicePortConfigVersion = u32;

// DevicePortConfig struct
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct DevicePortConfig {
    pub version: DevicePortConfigVersion,
    pub key: String,
    pub time_priority: DateTime<Utc>,
    pub state: DPCState,
    pub sha_file: String,
    pub sha_value: Option<Vec<u8>>,
    #[serde(flatten)]
    pub test_results: TestResults,
    #[serde(rename = "LastIPAndDNS")]
    pub last_ip_and_dns: DateTime<Utc>,
    pub ports: Vec<NetworkPortConfig>,
}

// DevicePortConfigList struct
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DevicePortConfigList {
    pub current_index: i32,
    pub port_config_list: Option<Vec<DevicePortConfig>>,
}

// NetworkPortConfig struct
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkPortConfig {
    pub if_name: String,
    #[serde(rename = "USBAddr")]
    pub usb_addr: String,
    #[serde(rename = "PCIAddr")]
    pub pci_addr: String,
    #[serde(rename = "Phylabel")]
    pub phy_label: String,
    #[serde(rename = "Logicallabel")]
    pub logical_label: String,
    pub alias: String,
    #[serde(rename = "NetworkUUID")]
    pub network_uuid: Uuid,
    pub is_mgmt: bool,
    pub is_l3_port: bool,
    pub cost: u8,
    #[serde(flatten)]
    pub dhcp_config: DhcpConfig,
    #[serde(flatten)]
    pub proxy_config: ProxyConfig,
    #[serde(flatten)]
    pub l2_link_config: L2LinkConfig,
    pub wireless_cfg: WirelessConfig,
    #[serde(flatten)]
    pub test_results: TestResults,
}

// DhcpConfig struct
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct DhcpConfig {
    pub addr_subnet: String,
    pub dhcp: DhcpType,
    #[serde(rename = "DNSServers")]
    pub dns_servers: Option<Vec<IpAddr>>,
    pub domain_name: String,
    pub gateway: String,
    #[serde(rename = "NTPServer")]
    pub ntp_server: String,
    #[serde(rename = "Type")]
    pub dhcp_type: NetworkType,
}