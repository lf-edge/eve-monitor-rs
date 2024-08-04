use std::{clone, net::IpAddr};

use crate::ipc::eve_types::NetworkPortStatus;
use macaddr::{MacAddr, MacAddr8};

pub struct NetworkStatus {
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkInterface {
    pub name: String,
    pub is_mgmt: bool,
    pub ipv4: Option<Vec<IpAddr>>,
    pub ipv6: Option<Vec<IpAddr>>,
    pub routes: Option<Vec<IpAddr>>,
    pub mac: MacAddr,
    pub ntp_server: Option<IpAddr>,
    pub up: bool,
}

pub fn list() -> Option<Vec<NetworkInterface>> {
    Some(vec![NetworkInterface {
        name: "eth0".to_string(),
        is_mgmt: true,
        ipv4: None,
        ipv6: None,
        routes: None,
        mac: MacAddr::V8(MacAddr8::nil()),
        ntp_server: None,
        up: true,
    }])
}

impl From<NetworkPortStatus> for NetworkInterface {
    fn from(port: NetworkPortStatus) -> Self {
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

        NetworkInterface {
            name: port.if_name,
            ipv4,
            ipv6,
            is_mgmt: port.is_mgmt,
            routes: port.default_routers,
            mac: port.mac_addr,
            ntp_server: port.ntp_server,
            up: port.up,
        }
    }
}
impl From<&NetworkPortStatus> for NetworkInterface {
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

        NetworkInterface {
            name: port.if_name.clone(),
            ipv4: ipv4,
            ipv6: ipv6,
            is_mgmt: port.is_mgmt,
            routes: port.default_routers.clone(),
            mac: port.mac_addr,
            ntp_server: port.ntp_server,
            up: port.up,
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
        let network_interface = NetworkInterface::from(port);
        let addresses = network_interface.ipv4.unwrap();

        assert_eq!(network_interface.name, "eth1");
        assert_eq!(network_interface.is_mgmt, true);
        assert_eq!(addresses.len(), 3);
        assert_eq!(
            network_interface.mac.as_bytes(),
            &[0x52, 0x54, 0x00, 0x12, 0x34, 0x57]
        );
        assert_eq!(network_interface.routes.unwrap().len(), 2);
        // check all the addresses
        assert_eq!(addresses[0], IpAddr::V4(Ipv4Addr::new(192, 168, 2, 10)));
        assert_eq!(
            addresses[1],
            IpAddr::from_str("fec0::21b8:b579:8b9c:3cda").unwrap()
        );
        assert_eq!(
            addresses[2],
            IpAddr::from_str("fe80::6f27:5660:de21:d553").unwrap()
        );
    }
}
