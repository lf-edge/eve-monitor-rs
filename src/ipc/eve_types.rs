use base64::Engine;
use chrono::DateTime;
use chrono::Utc;
use macaddr::MacAddr;
use macaddr::MacAddr6;
use macaddr::MacAddr8;
use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::default;
use std::net::IpAddr;
use strum::Display;
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
    #[serde(rename = "CellularV2")]
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

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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

fn serialize_ipaddr<S>(ip: &Option<IpAddr>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match ip {
        Some(ip) => ip.to_string().serialize(serializer),
        None => "".serialize(serializer),
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
    #[serde(rename = "DNSServers")]
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
    #[serde(rename = "pubsub-large-ProxyCertPEM")]
    pub proxy_cert_pem: Option<Vec<Vec<u8>>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct L2LinkConfig {
    l2_type: L2LinkType,
    #[serde(rename = "VLAN")]
    vlan: Option<VLANConfig>,
    bond: Option<BondConfig>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct TestResults {
    last_failed: DateTime<Utc>,
    last_succeeded: DateTime<Utc>,
    last_error: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct WirelessStatus {
    w_type: WirelessType,
    cellular: WwanNetworkStatus,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ProxyEntry {
    #[serde(rename = "type")]
    pub proxy_type: NetworkProxyType,
    pub server: String,
    pub port: u32,
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
    #[serde(rename = "SSID")]
    pub ssid: String,
    pub key_scheme: WifiKeySchemeType,
    pub identity: String, // to be deprecated, use CipherBlockStatus instead
    pub password: String, // to be deprecated, use CipherBlockStatus instead
    pub priority: i32,
    #[serde(flatten)]
    pub cipher_block_status: CipherBlockStatus,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CipherBlockStatus {
    #[serde(rename = "CipherBlockID")]
    pub cipher_block_id: String,
    #[serde(rename = "CipherContextID")]
    pub cipher_context_id: String,
    pub initial_value: Option<String>, //Vec<u8>,
    #[serde(rename = "pubsub-large-CipherData")]
    pub cipher_data: Option<String>, //Vec<u8>,
    pub clear_text_hash: Option<String>, //Vec<u8>,
    pub is_cipher: bool,
    pub cipher_context: Option<CipherContext>,
    #[serde(flatten)]
    pub error_and_time: ErrorAndTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CipherContext {
    // Define fields here
}

#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum WifiKeySchemeType {
    KeySchemeNone = 0,
    KeySchemeWpaPsk = 1,
    KeySchemeWpaEap = 2,
    KeySchemeOther = 3,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DeprecatedCellConfig {
    #[serde(rename = "APN")]
    pub apn: String,
    pub probe_addr: String,
    pub disable_probe: String,
    pub location_tracking: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WwanNetworkStatus {
    pub logical_label: String,
    pub phys_addrs: WwanPhysAddrs,
    pub module: WwanCellModule,
    pub sim_cards: Option<Vec<WwanSimCard>>,
    pub config_error: String,
    pub probe_error: String,
    pub current_provider: WwanProvider,
    pub visible_providers: Option<Vec<WwanProvider>>,
    pub current_rats: Option<Vec<WwanRAT>>,
    pub connected_at: u64,
    #[serde(rename = "IPSettings")]
    pub ip_settings: WwanIPSettings,
    pub location_tracking: bool,
}

fn ip_empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<IpAddr>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s.parse().map_err(serde::de::Error::custom)?))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WwanIPSettings {
    pub address: Option<GoIpNetwork>,
    #[serde(deserialize_with = "ip_empty_string_as_none")]
    pub gateway: Option<IpAddr>,
    #[serde(rename = "DNSServers")]
    pub dns_servers: Option<Vec<IpAddr>>,
    #[serde(rename = "MTU")]
    pub mtu: u16,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WwanPhysAddrs {
    // Define fields here
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WwanCellModule {
    // Define fields here
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WwanSimCard {
    // Define fields here
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WwanProvider {
    // Define fields here
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum WwanRAT {
    WwanRATUnspecified,
    WwanRATGSM,
    WwanRATUMTS,
    WwanRATLTE,
    WwanRAT5GNR,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CellNetPortConfig {
    pub access_points: Option<Vec<CellularAccessPoint>>,
    pub probe: WwanProbe,
    pub location_tracking: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WwanProbe {
    disable: bool,
    // IP/FQDN address to periodically probe to determine connection status.
    address: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum WwanAuthProtocol {
    None,
    Pap,
    Chap,
    PapChap,
}

fn deserialize_auth_protocol<'de, D>(deserializer: D) -> Result<WwanAuthProtocol, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    match s.as_str() {
        "" => Ok(WwanAuthProtocol::None),
        "pap" => Ok(WwanAuthProtocol::Pap),
        "chap" => Ok(WwanAuthProtocol::Chap),
        "pap-and-chap" => Ok(WwanAuthProtocol::PapChap),
        _ => Err(serde::de::Error::custom(format!(
            "Unknown auth protocol: {}",
            s
        ))),
    }
}

fn serialize_auth_protocol<S>(value: &WwanAuthProtocol, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = match value {
        WwanAuthProtocol::None => "".to_string(),
        WwanAuthProtocol::Pap => "pap".to_string(),
        WwanAuthProtocol::Chap => "chap".to_string(),
        WwanAuthProtocol::PapChap => "pap-and-chap".to_string(),
    };
    serializer.serialize_str(&s)
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CellularAccessPoint {
    pub key: String, // SIM card slot to which this configuration applies.
    // 0 - unspecified (apply to currently activated or the only available)
    // 1 - config for SIM card in the first slot
    // 2 - config for SIM card in the second slot
    // etc.
    pub sim_slot: u8,
    // If true, then this configuration is currently activated.
    pub activated: bool,
    // Access Point Network
    pub apn: String,
    // Authentication protocol used by the network.
    #[serde(
        deserialize_with = "deserialize_auth_protocol",
        serialize_with = "serialize_auth_protocol"
    )]
    pub auth_protocol: WwanAuthProtocol,
    // EncryptedCredentials : encrypted username and password.
    pub encrypted_credentials: CipherBlockStatus,
    // The set of cellular network operators that modem should preferably try to register
    // and connect into.
    // Network operator should be referenced by PLMN (Public Land Mobile Network) code.
    pub preferred_plmns: Vec<String>,
    // The list of preferred Radio Access Technologies (RATs) to use for connecting
    // to the network.
    pub preferred_rats: Vec<WwanRAT>,
    // If true, then modem will avoid connecting to networks with roaming.
    pub forbid_roaming: bool,
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
    #[serde(rename = "IPTargets")]
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
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WirelessConfig {
    #[serde(rename = "WType")]
    pub w_type: WirelessType,
    #[serde(rename = "CellularV2")]
    pub cellular_v2: Option<CellNetPortConfig>,
    pub wifi: Option<Vec<WifiConfig>>,
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

impl DevicePortConfig {
    pub fn get_port_by_name(&self, name: &str) -> Option<&NetworkPortConfig> {
        self.ports.iter().find(|npc| npc.if_name == name)
    }
    pub fn get_port_by_name_mut(&mut self, name: &str) -> Option<&mut NetworkPortConfig> {
        self.ports.iter_mut().find(|npc| npc.if_name == name)
    }

    // create new DPC with the given key based on the current DPC
    pub fn to_new_dpc_with_key(&self, key: &str) -> DevicePortConfig {
        DevicePortConfig {
            version: self.version,
            key: key.to_string(),
            // set current time as time_priority
            time_priority: Utc::now(),
            // TODO: is this correct?
            state: DPCState::None,
            // TODO: not sure what to do with sha_file and sha_value
            sha_file: self.sha_file.clone(),
            sha_value: self.sha_value.clone(),
            test_results: TestResults::default(),
            last_ip_and_dns: DateTime::default(),
            ports: self.ports.clone(),
        }
    }

    // pub fn update_or_insert_port(&mut self, port: NetworkPortConfig) {
    //     if let Some(p) = self.get_port_by_name_mut(&port.if_name) {
    //         *p = port;
    //     } else {
    //         self.ports.push(port);
    //     }
    // }
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

    pub fn get_current_dpc_ref(&self) -> Option<&DevicePortConfig> {
        self.port_config_list
            .as_ref()
            .and_then(|list| list.get(self.current_index as usize))
    }

    pub fn get_current_dpc_mut(&mut self) -> Option<&mut DevicePortConfig> {
        self.port_config_list
            .as_mut()
            .and_then(|list| list.get_mut(self.current_index as usize))
    }

    pub fn get_current_dpc_key(&self) -> Option<&str> {
        self.get_current_dpc_ref().map(|dpc| dpc.key.as_str())
    }

    pub fn get_current_dpc_cloned(&self) -> Option<DevicePortConfig> {
        self.get_current_dpc_ref().map(|dpc| dpc.clone())
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
    pub shared_labels: Option<Vec<String>>,
    pub alias: String,
    #[serde(rename = "NetworkUUID")]
    pub network_uuid: Uuid,
    pub is_mgmt: bool,
    pub is_l3_port: bool,
    pub invalid_config: bool,
    pub cost: u8,
    #[serde(rename = "MTU")]
    pub mtu: u16,
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

impl NetworkPortConfig {
    pub fn is_dhcp(&self) -> bool {
        self.dhcp_config.dhcp == DhcpType::Client
    }
    pub fn is_static(&self) -> bool {
        self.dhcp_config.dhcp == DhcpType::Static
    }
    // change the type of the port to DHCP
    pub fn into_dhcp(mut self) -> Self {
        self.dhcp_config.dhcp = DhcpType::Client;
        // clean static ip fields
        self.dhcp_config.addr_subnet = String::new();
        self.dhcp_config.gateway = String::new();
        self.dhcp_config.domain_name = String::new();
        self.dhcp_config.ntp_server = None;
        self.dhcp_config.dns_servers = None;
        //TODO: what do we do with NetworkUUID?
        self
    }

    pub fn into_static(
        mut self,
        addr_subnet: String,
        gateway: String,
        domain_name: String,
        ntp_server: Option<IpAddr>,
        dns_servers: Option<Vec<IpAddr>>,
    ) -> Self {
        self.dhcp_config.dhcp = DhcpType::Static;
        self.dhcp_config.addr_subnet = addr_subnet;
        self.dhcp_config.gateway = gateway;
        self.dhcp_config.domain_name = domain_name;
        self.dhcp_config.ntp_server = ntp_server;
        self.dhcp_config.dns_servers = dns_servers;
        self
    }

    pub fn to_dhcp(&mut self) {
        self.dhcp_config.dhcp = DhcpType::Client;
        // clean static ip fields
        self.dhcp_config.addr_subnet = String::new();
        self.dhcp_config.gateway = String::new();
        self.dhcp_config.domain_name = String::new();
        self.dhcp_config.ntp_server = None;
        self.dhcp_config.dns_servers = None;
    }

    pub fn to_static(
        &mut self,
        addr_subnet: String,
        gateway: String,
        domain_name: String,
        ntp_server: Option<IpAddr>,
        dns_servers: Option<Vec<IpAddr>>,
    ) {
        self.dhcp_config.dhcp = DhcpType::Static;
        self.dhcp_config.addr_subnet = addr_subnet;
        self.dhcp_config.gateway = gateway;
        self.dhcp_config.domain_name = domain_name;
        self.dhcp_config.ntp_server = ntp_server;
        self.dhcp_config.dns_servers = dns_servers;
    }
}

// DhcpConfig struct
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DhcpConfig {
    pub dhcp: DhcpType,
    pub addr_subnet: String,
    pub gateway: String,
    pub domain_name: String,
    #[serde(
        rename = "NTPServer",
        deserialize_with = "deserialize_ipaddr",
        serialize_with = "serialize_ipaddr"
    )]
    pub ntp_server: Option<IpAddr>,
    #[serde(rename = "DNSServers")]
    pub dns_servers: Option<Vec<IpAddr>>,
    #[serde(rename = "Type")]
    pub dhcp_type: NetworkType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorDescription {
    pub error: String,
    pub error_time: DateTime<Utc>,
    pub error_severity: ErrorSeverity,
    pub error_retry_condition: String,
    pub error_entities: Option<Vec<ErrorEntity>>,
}

#[repr(i32)]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
pub enum ErrorSeverity {
    Unspecified = 0,
    Notice = 1,
    Warning = 2,
    Error = 3,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorEntity {
    pub entity_type: ErrorEntityType,
    #[serde(rename = "EntityID")]
    pub entity_id: String,
}

#[repr(i32)]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicalIOAdapterList {
    pub initialized: bool,
    pub adapter_list: Vec<PhysicalIOAdapter>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicalIOAdapter {
    pub ptype: PhyIoType,
    pub phylabel: String,
    pub phyaddr: PhysicalAddress,
    pub logicallabel: String,
    pub assigngrp: String,
    pub parentassigngrp: String,
    pub usage: PhyIoMemberUsage,
    pub usage_policy: PhyIOUsagePolicy,
    pub vfs: VFList,
    pub cbattr: Option<std::collections::HashMap<String, String>>,
}

#[repr(i32)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum PhyIoType {
    PhyIoTypeNoop = 0,
    PhyIoTypeNetEth = 1,
    PhyIoTypeUSB = 2,
    PhyIoTypeCOM = 3,
    PhyIoTypeAudio = 4,
    PhyIoTypeNetWLAN = 5,
    PhyIoTypeNetWWAN = 6,
    PhyIoTypeHDMI = 7,
    PhyIoTypeNVMEStorage = 9,
    PhyIoTypeSATAStorage = 10,
    PhyIoTypeNetEthPF = 11,
    PhyIoTypeNetEthVF = 12,
    PhyIoTypeUSBController = 13,
    PhyIoTypeUSBDevice = 14,
    PhyIoTypeCAN = 15,
    PhyIoTypeVCAN = 16,
    PhyIoTypeLCAN = 17,
    PhyIoTypeOther = 255,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicalAddress {
    pub pci_long: String,
    pub ifname: String,
    pub serial: String,
    pub irq: String,
    pub ioports: String,
    pub usb_addr: String,
    pub usb_product: String,
    pub unknown_type: String,
}

#[repr(i32)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum PhyIoMemberUsage {
    PhyIoUsageNone = 0,
    PhyIoUsageMgmtAndApps = 1,
    PhyIoUsageShared = 2,
    PhyIoUsageDedicated = 3,
    PhyIoUsageDisabled = 4,
    PhyIoUsageMgmtOnly = 5,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PhyIOUsagePolicy {
    pub free_uplink: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct VFList {
    pub count: u8,
    pub data: Option<Vec<EthVF>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct EthVF {
    pub index: u8,
    pub pci_long: String,
    pub mac: String,
    pub vlan_id: u16,
}

// application related types
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AppInstanceStatus {
    #[serde(rename = "UUIDandVersion")]
    pub uuid_and_version: UUIDandVersion,
    pub display_name: String,
    pub domain_name: String,
    pub activated: bool,
    pub activate_inprogress: bool,
    pub fixed_resources: VmConfig,
    pub volume_ref_status_list: Vec<VolumeRefStatus>,
    #[serde(skip)]
    pub app_net_adapters: Vec<AppNetAdapterStatus>,
    pub boot_time: String, // Replace with a suitable time type
    #[serde(skip)]
    pub io_adapter_list: Vec<IoAdapter>,
    pub restart_inprogress: Inprogress,
    pub restart_started_at: String, // Replace with a suitable time type
    pub purge_inprogress: Inprogress,
    pub purge_started_at: String, // Replace with a suitable time type
    pub state: SwState,
    pub missing_network: bool,
    pub missing_memory: bool,
    #[serde(flatten)]
    pub error_and_time_with_source: ErrorAndTimeWithSource,
    pub start_time: String, // Replace with a suitable time type
    #[serde(skip)]
    pub snap_status: SnapshottingStatus,
    pub mem_overhead: u64,
}

#[repr(u8)]
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Copy, Display)]
pub enum SwState {
    Initial = 100,
    ResolvingTag = 101,
    ResolvedTag = 102,
    Downloading = 103,
    Downloaded = 104,
    Verifying = 105,
    Verified = 106,
    Loading = 107,
    Loaded = 108,
    CreatingVolume = 109,
    CreatedVolume = 110,
    Installed = 111,
    AwaitNetworkInstance = 112,
    StartDelayed = 113,
    Booting = 114,
    Running = 115,
    Pausing = 116,
    Paused = 117,
    Halting = 118,
    Halted = 119,
    Broken = 120,
    Unknown = 121,
    Pending = 122,
    Scheduling = 123,
    Failed = 124,
    MaxState = 125,
}

impl SwState {
    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct UUIDandVersion {
    #[serde(rename = "UUID")]
    pub uuid: Uuid,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct VmConfig {
    pub kernel: String,
    pub ramdisk: String,
    pub memory: i32,
    pub max_mem: i32,
    #[serde(rename = "VCpus")]
    pub vcpus: i32,
    pub max_cpus: i32,
    pub root_dev: String,
    pub extra_args: String,
    pub boot_loader: String,
    #[serde(rename = "CPUs")]
    pub cpus: String,
    pub device_tree: String,
    pub dt_dev: Option<Vec<String>>,
    #[serde(rename = "IRQs")]
    pub irqs: Option<Vec<i32>>,
    #[serde(rename = "IOMem")]
    pub iomem: Option<Vec<String>>,
    pub virtualization_mode: VmMode,
    pub enable_vnc: bool,
    pub vnc_display: u32,
    pub vnc_passwd: String,
    #[serde(rename = "CPUsPinned")]
    pub cpus_pinned: bool,
    #[serde(rename = "VMMMaxMem")]
    pub vmm_max_mem: i32,
    #[serde(rename = "EnableVncShimVM")]
    pub enable_vnc_shim_vm: bool,
}

#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum VmMode {
    PV = 0,
    HVM = 1,
    Filler = 2,
    FML = 3,
    NoHyper = 4,
    Legacy = 5,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct VolumeRefStatus {
    #[serde(rename = "VolumeID")]
    pub volume_id: Uuid,
    pub generation_counter: i64,
    pub local_generation_counter: i64,
    #[serde(rename = "AppUUID")]
    pub app_uuid: Uuid,
    pub state: SwState,
    pub active_file_location: String,
    pub content_format: Format,
    pub read_only: bool,
    pub display_name: String,
    pub max_vol_size: u64,
    pub pending_add: bool,
    #[serde(rename = "WWN")]
    pub wwn: String,
    pub verify_only: bool,
    pub target: Target,
    pub custom_meta: String,
    pub reference_name: String,
    #[serde(flatten)]
    pub error_and_time_with_source: ErrorAndTimeWithSource,
}

#[repr(i32)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum Format {
    FmtUnknown = 0,
    RAW = 1,
    QCOW = 2,
    QCOW2 = 3,
    VHD = 4,
    VMDK = 5,
    OVA = 6,
    VHDX = 7,
    Container = 8,
    ISO = 9,
    PVC = 10,
}

#[repr(i32)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum Target {
    TgtUnknown = 0,
    Disk = 1,
    Kernel = 2,
    Initrd = 3,
    RamDisk = 4,
    AppCustom = 5,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorAndTimeWithSource {
    pub error_source_type: String,
    #[serde(flatten)]
    pub error_description: ErrorDescription,
}

#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum Inprogress {
    NotInprogress = 0,
    DownloadAndVerify = 1,
    BringDown = 2,
    RecreateVolumes = 3,
    BringUp = 4,
}

// Placeholder types for unknown ones
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct AppNetAdapterStatus {} // Replace with actual definition

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct IoAdapter {} // Replace with actual definition

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct SnapshottingStatus {} // Replace with actual definition

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EveOnboardingStatus {
    #[serde(rename = "DeviceUUID")]
    pub device_uuid: Uuid,
    pub hardware_model: String, // From controller
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EveVaultStatus {
    pub name: String,
    pub status: DataSecAtRestStatus,
    #[serde(rename = "PCRStatus")]
    pub pcr_status: PCRStatus,
    pub conversion_complete: bool,
    #[serde(rename = "MissmatchingPCRs")]
    pub missmatching_pcrs: Vec<i32>,
    #[serde(flatten)]
    pub error_and_time: ErrorAndTime, // Unknown type, skipped
}

#[repr(i32)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum DataSecAtRestStatus {
    DataSecAtRestUnknown = 0,  // Status is unknown
    DataSecAtRestDisabled = 1, // Enabled, but not being used
    DataSecAtRestEnabled = 2,  // Enabled, and used
    DataSecAtRestError = 4,    // Enabled, but encountered an error
}

#[repr(i32)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum PCRStatus {
    PcrUnknown = 0,  // Status is unknown
    PcrEnabled = 1,  // Enabled PCR
    PcrDisabled = 2, // Disabled PCR
}

type AppCount = u8;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct AppInstanceSummary {
    //#[serde(rename = "UUIDandVersion")]
    //pub uuid_and_version: UUIDandVersion,
    pub total_starting: AppCount, // Total number of apps starting/booting
    pub total_running: AppCount,  // Total number of apps in running state
    pub total_stopping: AppCount, // Total number of apps in halting state
    pub total_error: AppCount,    // Total number of apps in error state
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LedBlinkCounter {
    pub blink_counter: LedBlinkCount,
}

#[repr(u8)]
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
pub enum LedBlinkCount {
    LedBlinkUndefined = 0,
    LedBlinkWaitingForIP,
    LedBlinkConnectingToController,
    LedBlinkConnectedToController,
    LedBlinkOnboarded,
    LedBlinkRadioSilence,
    LedBlinkOnboardingFailure = 10,
    LedBlinkRespWithoutTLS = 12,
    LedBlinkRespWithoutOSCP,
    LedBlinkInvalidControllerCert,
    LedBlinkInvalidAuthContainer,
    LedBlinkInvalidBootstrapConfig,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EveNodeStatus {
    pub server: Option<String>,
    #[serde(deserialize_with = "zero_uuid_as_none")]
    pub node_uuid: Option<Uuid>,
    pub onboarded: bool,
    pub app_instance_summary: Option<AppInstanceSummary>,
}

fn zero_uuid_as_none<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s == "00000000-0000-0000-0000-000000000000" {
        Ok(None)
    } else {
        Ok(Some(Uuid::parse_str(&s).map_err(serde::de::Error::custom)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppsList {
    pub apps: Vec<AppInstanceStatus>,
}
