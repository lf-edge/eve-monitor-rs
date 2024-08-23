use std::{clone, net::IpAddr};

use crate::ipc::eve_types::{
    DhcpType, NetworkPortConfig, NetworkPortStatus, NetworkProxyType, ProxyEntry, WirelessType,
};
use macaddr::MacAddr;

pub struct NetworkStatus {
    pub interfaces: Vec<NetworkInterfaceStatus>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimStatus {
    pub apn: String,
    pub slot: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellularStatus {
    sims: Vec<SimStatus>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WiFiStatus {
    pub ssid: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkType {
    Ethernet,
    WiFi(WiFiStatus),
    Cellular(CellularStatus),
}

impl NetworkType {
    pub fn to_string(&self) -> String {
        match self {
            NetworkType::Ethernet => "Ethernet".to_string(),
            NetworkType::WiFi(_) => "WiFi".to_string(),
            NetworkType::Cellular(_) => "Cellular".to_string(),
        }
    }
}

pub enum IpConfig {
    Static {
        ip: Vec<IpAddr>,
        gw: IpAddr,
        ntp_servers: Option<Vec<IpAddr>>,
        routes: Option<Vec<IpAddr>>,
    },
    Dhcp,
}

pub enum PhyConfig {
    Ethernet { mtu: u32 },
    WiFi { ssid: String, password: String },
    Cellular { apn: String, slot: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProxyHost {
    server: String,
    port: u32,
}

impl ProxyHost {
    pub fn to_url(&self) -> String {
        format!("{}:{}", self.server, self.port)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProxyConfig {
    None,
    Pac {
        url: String,
    },
    Manual {
        http: Option<ProxyHost>,
        https: Option<ProxyHost>,
        ftp: Option<ProxyHost>,
        socks: Option<ProxyHost>,
    },
    Wad {
        url: String,
    },
}

impl ProxyConfig {
    fn is_manual(&self) -> bool {
        if let ProxyConfig::Manual {
            http,
            https,
            ftp,
            socks,
        } = self
        {
            http.is_some() || https.is_some() || ftp.is_some() || socks.is_some()
        } else {
            false
        }
    }
}

impl From<&crate::ipc::eve_types::ProxyConfig> for ProxyConfig {
    fn from(port_proxy_config: &crate::ipc::eve_types::ProxyConfig) -> Self {
        let iface_proxy_config = if port_proxy_config.network_proxy_enable {
            ProxyConfig::Wad {
                url: port_proxy_config.network_proxy_url.clone(),
            }
        } else if !port_proxy_config.pacfile.is_empty() {
            ProxyConfig::Pac {
                url: port_proxy_config.pacfile.clone(),
            }
        } else if let Some(proxies) = &port_proxy_config.proxies {
            let mut http_proxy = None;
            let mut https_proxy = None;
            let mut ftp_proxy = None;
            let mut socks_proxy = None;

            proxies.iter().for_each(|proxy| match proxy.proxy_type {
                NetworkProxyType::HTTP => {
                    http_proxy = Some(ProxyHost {
                        server: proxy.server.clone(),
                        port: proxy.port,
                    });
                }
                NetworkProxyType::SOCKS => {
                    socks_proxy = Some(ProxyHost {
                        server: proxy.server.clone(),
                        port: proxy.port,
                    });
                }
                NetworkProxyType::FTP => {
                    ftp_proxy = Some(ProxyHost {
                        server: proxy.server.clone(),
                        port: proxy.port,
                    });
                }
                NetworkProxyType::HTTPS => {
                    https_proxy = Some(ProxyHost {
                        server: proxy.server.clone(),
                        port: proxy.port,
                    });
                }
                NetworkProxyType::NOPROXY => {}
                NetworkProxyType::LAST => {}
            });

            let manual_proxies = ProxyConfig::Manual {
                http: http_proxy,
                https: https_proxy,
                ftp: ftp_proxy,
                socks: socks_proxy,
            };

            if manual_proxies.is_manual() {
                manual_proxies
            } else {
                ProxyConfig::None
            }
        } else {
            ProxyConfig::None
        };
        iface_proxy_config
    }
}

impl Default for PhyConfig {
    fn default() -> Self {
        PhyConfig::Ethernet { mtu: 1500 }
    }
}

pub struct InterfaceConfig {
    pub name: String,
    pub ip_config: IpConfig,
    pub phy_config: PhyConfig,
    pub proxy_config: ProxyConfig,
    pub proxy_certificate: Option<String>,
}

//TODO: convert to enum and create a separate struct for common fields
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkInterfaceStatus {
    pub name: String,
    pub is_mgmt: bool,
    pub ipv4: Option<Vec<IpAddr>>,
    pub ipv6: Option<Vec<IpAddr>>,
    pub routes: Option<Vec<IpAddr>>,
    pub mac: MacAddr,
    pub ntp_servers: Option<Vec<IpAddr>>,
    pub up: bool,
    pub media: NetworkType,
    pub dns: Option<Vec<IpAddr>>,
    pub gw: Option<IpAddr>,
    pub is_dhcp: bool,
    pub proxy_config: ProxyConfig,
    pub domain: Option<String>,
    pub cost: u8,
}

impl From<&NetworkPortStatus> for NetworkInterfaceStatus {
    fn from(port: &NetworkPortStatus) -> Self {
        // parse address list
        let ipv4 = port.addr_info_list.as_ref().map(|addr_info_list| {
            addr_info_list
                .iter()
                .filter(|addr_info| addr_info.addr.is_ipv4())
                .map(|addr_info| addr_info.addr)
                .collect()
        });

        let ipv6 = port.addr_info_list.as_ref().map(|addr_info_list| {
            addr_info_list
                .iter()
                .filter(|addr_info| addr_info.addr.is_ipv6())
                .map(|addr_info| addr_info.addr)
                .collect()
        });

        // set media type
        let media = match port.wireless_cfg.w_type {
            WirelessType::None => NetworkType::Ethernet,
            WirelessType::Wifi => NetworkType::WiFi(WiFiStatus {
                //FIXME: why we have a Vec of WifiConfig?
                ssid: port
                    .wireless_cfg
                    .wifi
                    .as_ref()
                    .and_then(|w| Some(w[0].ssid.clone())),
            }),
            WirelessType::Cellular => NetworkType::Cellular(CellularStatus {
                // A modem can have multiple sims
                sims: port
                    .wireless_cfg
                    .cellular_v2
                    .as_ref()
                    .and_then(|c| {
                        c.access_points.as_ref().and_then(|a| {
                            Some(
                                a.iter()
                                    .map(|s| SimStatus {
                                        apn: s.apn.clone(),
                                        slot: u32::from(s.sim_slot),
                                    })
                                    .collect(),
                            )
                        })
                    })
                    .unwrap(),
            }),
            _ => NetworkType::Ethernet,
        };

        let is_dhcp = port.dhcp == DhcpType::Client;

        // collect DNS servers
        let dns = port.dns_servers.as_ref().map(|dns_servers| {
            dns_servers
                .iter()
                .map(|dns_server| dns_server.clone())
                .collect()
        });

        // take the first default router as the gateway
        let gw = port
            .default_routers
            .as_ref()
            .and_then(|routers| routers.first().cloned());

        NetworkInterfaceStatus {
            name: port.if_name.clone(),
            ipv4: ipv4,
            ipv6: ipv6,
            is_mgmt: port.is_mgmt,
            routes: port.default_routers.clone(),
            mac: port.mac_addr,
            ntp_servers: port.ntp_servers.clone(),
            up: port.up,
            media,
            dns,
            gw,
            is_dhcp,
            cost: port.cost,
            domain: if port.domain_name.is_empty() {
                None
            } else {
                Some(port.domain_name.clone())
            },
            proxy_config: (&port.proxy_config).into(),
        }
    }
}
#[cfg(test)]
mod tests {
    use std::{net::Ipv4Addr, str::FromStr};

    use serde_json::from_value;
    use serde_json::json;

    use super::*;
    fn get_network_port_status() -> NetworkPortStatus {
        let json = json!({
            "IfName": "eth1",
            "Phylabel": "eth1",
            "Logicallabel": "eth1",
            "Alias": "",
            "IsMgmt": true,
            "IsL3Port": true,
            "Cost": 0,
            "Dhcp": 4,
            "Type": 0,
            "Subnet": {
                "IP": "192.168.2.0",
                "Mask": "////AA=="
            },
            "NtpServer": "",
            "DomainName": "",
            "DNSServers": [
                "192.168.2.3"
            ],
            "NtpServers": null,
            "AddrInfoList": [
                {
                    "Addr": "192.168.2.10",
                    "Geo": {
                        "ip": "",
                        "hostname": "",
                        "city": "",
                        "region": "",
                        "country": "",
                        "loc": "",
                        "org": "",
                        "postal": ""
                    },
                    "LastGeoTimestamp": "0001-01-01T00:00:00Z"
                },
                {
                    "Addr": "fec0::21b8:b579:8b9c:3cda",
                    "Geo": {
                        "ip": "",
                        "hostname": "",
                        "city": "",
                        "region": "",
                        "country": "",
                        "loc": "",
                        "org": "",
                        "postal": ""
                    },
                    "LastGeoTimestamp": "0001-01-01T00:00:00Z"
                },
                {
                    "Addr": "fe80::6f27:5660:de21:d553",
                    "Geo": {
                        "ip": "",
                        "hostname": "",
                        "city": "",
                        "region": "",
                        "country": "",
                        "loc": "",
                        "org": "",
                        "postal": ""
                    },
                    "LastGeoTimestamp": "0001-01-01T00:00:00Z"
                }
            ],
            "Up": true,
            "MacAddr": "UlQAEjRX",
            "DefaultRouters": [
                "192.168.2.2",
                "fe80::2"
            ],
            "MTU": 1500,
            "WirelessCfg": {
                "WType": 0,
                "CellularV2": {
                    "AccessPoints": null,
                    "Probe": {
                        "Disable": false,
                        "Address": ""
                    },
                    "LocationTracking": false
                },
                "Wifi": null,
                "Cellular": null
            },
            "WirelessStatus": {
                "WType": 0,
                "Cellular": {
                    "LogicalLabel": "",
                    "PhysAddrs": {
                        "Interface": "",
                        "USB": "",
                        "PCI": "",
                        "Dev": ""
                    },
                    "Module": {
                        "Name": "",
                        "IMEI": "",
                        "Model": "",
                        "Manufacturer": "",
                        "Revision": "",
                        "ControlProtocol": "",
                        "OpMode": ""
                    },
                    "SimCards": null,
                    "ConfigError": "",
                    "ProbeError": "",
                    "CurrentProvider": {
                        "PLMN": "",
                        "Description": "",
                        "CurrentServing": false,
                        "Roaming": false,
                        "Forbidden": false
                    },
                    "VisibleProviders": null,
                    "CurrentRATs": null,
                    "ConnectedAt": 0,
                    "IPSettings": {
                        "Address": null,
                        "Gateway": "",
                        "DNSServers": null,
                        "MTU": 0
                    },
                    "LocationTracking": false
                }
            },
            "Proxies": null,
            "Exceptions": "",
            "Pacfile": "",
            "NetworkProxyEnable": false,
            "NetworkProxyURL": "",
            "WpadURL": "",
            "pubsub-large-ProxyCertPEM": null,
            "L2Type": 0,
            "VLAN": {
                "ParentPort": "",
                "ID": 0
            },
            "Bond": {
                "AggregatedPorts": null,
                "Mode": 0,
                "LacpRate": 0,
                "MIIMonitor": {
                    "Enabled": false,
                    "Interval": 0,
                    "UpDelay": 0,
                    "DownDelay": 0
                },
                "ARPMonitor": {
                    "Enabled": false,
                    "Interval": 0,
                    "IPTargets": null
                }
            },
            "LastFailed": "2024-07-22T06:27:12.052879635Z",
            "LastSucceeded": "0001-01-01T00:00:00Z",
            "LastError": "All attempts to connect to https://zedcloud.alpha.zededa.net/api/v2/edgedevice/ping failed: interface eth1: no DNS server available"
        });

        from_value(json).unwrap()
    }

    #[test]
    fn test_from() {
        let port = get_network_port_status();
        let network_interface = NetworkInterfaceStatus::from(&port);
        let ipv4_addresses = network_interface.ipv4.unwrap();
        let ipv6_addresses = network_interface.ipv6.unwrap();

        assert_eq!(network_interface.name, "eth1");
        assert_eq!(network_interface.is_mgmt, true);
        assert_eq!(ipv4_addresses.len(), 1);
        assert_eq!(ipv6_addresses.len(), 2);
        assert_eq!(
            network_interface.mac.as_bytes(),
            &[0x52, 0x54, 0x00, 0x12, 0x34, 0x57]
        );
        assert_eq!(network_interface.routes.unwrap().len(), 2);
        // check all the addresses
        assert_eq!(
            ipv4_addresses[0],
            IpAddr::V4(Ipv4Addr::new(192, 168, 2, 10))
        );
        assert_eq!(
            ipv6_addresses[0],
            IpAddr::from_str("fec0::21b8:b579:8b9c:3cda").unwrap()
        );
        assert_eq!(
            ipv6_addresses[1],
            IpAddr::from_str("fe80::6f27:5660:de21:d553").unwrap()
        );
    }
}
