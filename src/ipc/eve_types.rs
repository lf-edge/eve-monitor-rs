use base64::Engine;
use chrono::DateTime;
use chrono::Utc;
use macaddr::MacAddr;
use macaddr::MacAddr6;
use macaddr::MacAddr8;
use serde::de;
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
    pub mask: Option<String>, // base64 encoded prefix
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

pub fn deserialize_mac<'de, D>(deserializer: D) -> Result<MacAddr, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(de::Error::custom)?;

    match bytes.len() {
        6 => {
            let array: [u8; 6] = bytes
                .try_into()
                .map_err(|_| de::Error::custom("invalid byte array length"))?;
            let mac = MacAddr::from(MacAddr6::from(array));
            Ok(mac)
        }
        8 => {
            let array: [u8; 8] = bytes
                .try_into()
                .map_err(|_| de::Error::custom("invalid byte array length"))?;
            let mac = MacAddr::from(MacAddr8::from(array));
            Ok(mac)
        }
        _ => Err(de::Error::custom("invalid MAC address length")),
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
    pub addr_info_list: Option<Vec<AddrInfo>>,
    pub up: bool,
    #[serde(deserialize_with = "deserialize_mac", skip_serializing)]
    pub mac_addr: MacAddr,
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

/// NetworkPortStatus struct
/// Field names are confusing
/// 1. If network_proxy_enable is true, then use network_proxy_url is used to download .wpad file
/// 2. If network_proxy_enable is false, then one of the proxies from the proxies list is used
/// 3. Only one entry per proxy type  is possible in the proxies list
/// 4. If [ProxyConfig::pacfile] is used then proxy configuration is taken from the .pac file
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ProxyConfig {
    pub proxies: Option<Vec<ProxyEntry>>,
    pub exceptions: String,
    pub pacfile: String,
    pub network_proxy_enable: bool,
    #[serde(rename = "NetworkProxyURL")]
    pub network_proxy_url: String,
    #[serde(rename = "WpadURL")]
    pub wpad_url: String,
    pub proxy_cert_pem: Option<Vec<Vec<u8>>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct L2LinkConfig {
    l2_type: L2LinkType,
    vlan: Option<VLANConfig>,
    bond: Option<BondConfig>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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
    _cellular: WwanNetworkStatus,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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
    pub addr: IpAddr,
    pub geo: Option<IPInfo>,
    pub last_geo_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IPInfo {
    pub ip: String,
    pub hostname: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub loc: String,
    pub org: String,
    pub postal: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WifiConfig {
    // Define the fields
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DeprecatedCellConfig {
    // Define the fields
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct WwanNetworkStatus {
    // Define the fields
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CellNetPortConfig {
    #[serde(skip)]
    access_points: Vec<CellularAccessPoint>,
    probe: WwanProbe,
    location_tracking: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WwanProbe {
    disable: bool,
    // IP/FQDN address to periodically probe to determine connection status.
    address: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum L2LinkType {
    // L2LinkTypeNone : not an L2 link (used for physical network adapters).
    L2LinkTypeNone = 0,
    // L2LinkTypeVLAN : VLAN sub-interface
    L2LinkTypeVLAN = 1,
    // L2LinkTypeBond : Bond interface
    L2LinkTypeBond = 2,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct VLANConfig {
    parent_port: String,
    #[serde(rename = "ID")]
    id: u16,
}

#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
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

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BondMIIMonitor {
    pub down_delay: u32,
    pub enabled: bool,
    pub interval: u32,
    pub up_delay: u32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BondArpMonitor {
    pub enabled: bool,
    pub ip_targets: Option<String>,
    pub interval: u32,
}

#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum LacpRate {
    LacpRateUnspecified = 0,
    LacpRateSlow = 1,
    LacpRateFast = 2,
}

/// DhcpType enum
/// The name is confusing. Possible values are:
/// [NOOP, Static, None, Deprecated, Client]
/// but only [Client and Static] are used.
/// Corresponding values that can be used in PortConfigOverride.json
/// [0, 1, 2, 3, 4]
///
/// [Client] is the real DHCP client
/// [Static] is the static IP address
#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum DhcpType {
    NOOP = 0,
    Static = 1,
    None = 2,
    Deprecated = 3,
    /// DHCP client i.e. real DHCP client
    Client = 4,
}

// DPCState enum
#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
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
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
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
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
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
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum WirelessType {
    None = 0,
    Cellular = 1,
    Wifi = 2,
}

// WirelessConfig struct
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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

impl DevicePortConfigList {
    pub fn get_dpc_by_key(&self, key: &str) -> Option<&DevicePortConfig> {
        self.port_config_list
            .as_ref()
            .and_then(|list| list.iter().find(|dpc| dpc.key == key))
    }
}

// NetworkPortConfig struct
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DownloaderStatus {
    pub image_sha256: String,
    #[serde(rename = "DatastoreIDList")]
    pub datastore_id_list: Vec<Uuid>,
    pub target: String,
    pub name: String,
    pub ref_count: u32,
    pub last_use: DateTime<Utc>,
    pub expired: bool,
    #[serde(rename = "NameIsURL")]
    pub name_is_url: bool,
    pub state: SwState,
    pub reserved_space: u64,
    pub size: u64,
    pub total_size: i64,
    pub current_size: i64,
    pub progress: u32,
    pub mod_time: DateTime<Utc>,
    pub content_type: String,
    #[serde(flatten)]
    pub error_and_time: ErrorAndTime,
    pub retry_count: i32,
    pub orig_error: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorAndTime {
    #[serde(flatten)]
    pub error_description: ErrorDescription,
}

impl ErrorAndTime {
    pub fn is_error(&self) -> bool {
        !self.error_description.error.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorDescription {
    pub error: String,
    pub error_time: DateTime<Utc>,
    pub error_severity: ErrorSeverity,
    pub error_retry_condition: String,
    pub error_entities: Option<Vec<ErrorEntity>>,
}

#[repr(i32)]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
pub enum ErrorSeverity {
    Unspecified = 0,
    Notice = 1,
    Warning = 2,
    Error = 3,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorEntity {
    pub entity_type: ErrorEntityType,
    pub entity_id: String,
}

#[repr(i32)]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
pub enum ErrorEntityType {
    Unspecified = 0,
    BaseOs = 1,
    SystemAdapter = 2,
    Vault = 3,
    Attestation = 4,
    AppInstance = 5,
    Port = 6,
    Network = 7,
    NetworkInstance = 8,
    ContentTree = 9,
    ContentBlob = 10,
    Volume = 11,
}

#[repr(u8)]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
pub enum SwState {
    Initial = 100,
    ResolvingTag,
    ResolvedTag,
    Downloading,
    Downloaded,
    Verifying,
    Verified,
    Loading,
    Loaded,
    CreatingVolume,
    CreatedVolume,
    Installed,
    AwaitNetworkInstance,
    StartDelayed,
    Booting,
    Running,
    Pausing,
    Paused,
    Halting,
    Halted,
    Broken,
    Unknown,
    Pending,
    Scheduling,
    Failed,
    MaxState,
}
