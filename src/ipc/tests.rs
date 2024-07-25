use std::str::FromStr;

use super::*;
use eve_types::deserialize_mac;
use eve_types::BondConfig;
use eve_types::BondMode;
use eve_types::DPCState;
use eve_types::DeviceNetworkStatus;
use eve_types::DevicePortConfigList;
use eve_types::DhcpType;
use eve_types::GoIpNetwork;
use eve_types::LacpRate;
use eve_types::MIIMonitor;
use eve_types::NetworkPortStatus;
use eve_types::RadioSilence;
use eve_types::ResultData;
use eve_types::WirelessCfg;
use eve_types::WirelessType;
use macaddr::MacAddr;
use serde::de;
use serde::Deserialize;
use serde_json::from_value;
use serde_json::json;

#[test]
fn test_result_data() {
    let json_data = json!({
        "Key": "example_key",
        "LastError": "none",
        "LastFailed": "0001-01-01T00:00:00Z",
        "LastIPAndDNS": "0001-01-01T00:00:00Z",
        "LastSucceeded": "2024-07-20T06:44:00.588210162Z",
        "Ports": [],
        "ShaFile": "example_sha_file",
        "ShaValue": null,
        "State": 3,
        "TimePriority": "2024-07-20T06:44:00.588210162Z",
        "Version": 1,
    });

    let result: ResultData = from_value(json_data).unwrap();
    assert_eq!(result.key, "example_key");
    assert_eq!(result.last_error, "none");
    assert_eq!(result.state, 3);
}

#[test]
fn test_device_network_status() {
    let json_data = json!({
        "DPCKey": "lastresort",
        "Version": 1,
        "Testing": false,
        "State": 3,
        "CurrentIndex": 0,
        "RadioSilence": {
            "Imposed": false,
            "ChangeInProgress": false,
            "ChangeRequestedAt": "2024-07-20T06:43:50.477711902Z",
            "ConfigError": ""
        },
        "Ports": []
    });

    let result: DeviceNetworkStatus = from_value(json_data).unwrap();
    assert_eq!(result.dpc_key, "lastresort");
    assert_eq!(result.state, DPCState::Success);
    assert_eq!(result.testing, false);
}

#[test]
fn test_network_port_status() {
    let json_data = r#"
    {
        "IfName": "eth0",
        "Phylabel": "eth0",
        "Logicallabel": "eth0",
        "Alias": "",
        "IsMgmt": true,
        "IsL3Port": true,
        "Cost": 0,
        "Dhcp": 4,
        "Type": 0,
        "Subnet": {
            "IP": "192.168.1.0",
            "Mask": "////AA=="
        },
        "NtpServer": "",
        "DomainName": "",
        "DNSServers": [
            "192.168.1.3"
        ],
        "NtpServers": null,
        "AddrInfoList": [
            {
                "Addr": "192.168.1.10",
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
                "Addr": "fec0::6e70:6edc:1bd3:4119",
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
                "Addr": "fe80::ac64:4852:7b50:dc95",
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
        "MacAddr": "UlQAEjRW",
        "DefaultRouters": [
            "192.168.1.2",
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
        "LastFailed": "0001-01-01T00:00:00Z",
        "LastSucceeded": "2024-07-22T06:27:18.593306585Z",
        "LastError": ""
    }
    "#;

    let result: NetworkPortStatus = serde_json::from_str(&json_data).unwrap();
    assert_eq!(result.if_name, "eth0");
    assert_eq!(result.dhcp, DhcpType::Client);
    assert_eq!(
        result.subnet,
        GoIpNetwork {
            ip: "192.168.1.0".to_string(),
            mask: "////AA==".to_string()
        }
    );
}

#[test]
fn test_radio_silence() {
    let json_data = json!({
        "Imposed": false,
        "ChangeInProgress": false,
        "ChangeRequestedAt": "2024-07-20T06:43:50.477711902Z",
        "ConfigError": ""
    });

    let result: RadioSilence = from_value(json_data).unwrap();
    assert_eq!(result.imposed, false);
    assert_eq!(result.config_error, "");
}

#[test]
fn test_go_ip_network() {
    let json_data = json!({
        "IP": "192.168.1.0",
        "Mask": "////AA=="
    });

    let result: GoIpNetwork = from_value(json_data).unwrap();
    assert_eq!(result.ip, "192.168.1.0");
    assert_eq!(result.mask, "////AA==");
}

#[test]
fn test_wireless_cfg() {
    let json_data = r#"
    {
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
    }"#;

    let result: WirelessCfg = serde_json::from_str(&json_data).unwrap();
    assert_eq!(result.w_type, WirelessType::None);
}

#[test]
fn test_deserialize_miimonitor() {
    let json = r#"{
        "Enabled": true,
        "Interval": 5000,
        "UpDelay": 1000,
        "DownDelay": 2000
    }"#;

    let monitor: MIIMonitor = serde_json::from_str(json).unwrap();

    assert_eq!(monitor.enabled, true);
    assert_eq!(monitor.interval, 5000);
    assert_eq!(monitor.up_delay, 1000);
    assert_eq!(monitor.down_delay, 2000);
}

#[test]
fn test_serialize_miimonitor() {
    let monitor = MIIMonitor {
        enabled: true,
        interval: 5000,
        up_delay: 1000,
        down_delay: 2000,
    };

    let json = serde_json::to_string(&monitor).unwrap();

    let expected_json = r#"{"Enabled":true,"Interval":5000,"UpDelay":1000,"DownDelay":2000}"#;

    assert_eq!(json, expected_json);
}
#[test]
fn test_bond_config_deserialization() {
    let json = r#"
        {
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
        }
    "#;

    let bond_config: BondConfig = serde_json::from_str(json).unwrap();

    assert_eq!(bond_config.aggregated_ports.is_none(), true);
    assert_eq!(bond_config.mode, BondMode::BondModeUnspecified);
    assert_eq!(bond_config.lacp_rate, LacpRate::LacpRateUnspecified);
    // assert_eq!(bond_config.mii_monitor.enabled, false);
    // assert_eq!(bond_config.mii_monitor.interval, 0);
    // assert_eq!(bond_config.mii_monitor.up_delay, 0);
    // assert_eq!(bond_config.mii_monitor.down_delay, 0);
    // assert_eq!(bond_config.arp_monitor.enabled, false);
    // assert_eq!(bond_config.arp_monitor.interval, 0);
    assert_eq!(bond_config.arp_monitor.ip_targets.is_none(), true);
}

#[test]
fn test_device_network_status_full() {
    let json_data = r#"
    {
        "DPCKey": "lastresort",
        "Version": 1,
        "Testing": false,
        "State": 3,
        "CurrentIndex": 0,
        "RadioSilence": {
            "Imposed": false,
            "ChangeInProgress": false,
            "ChangeRequestedAt": "2024-07-22T06:27:09.366225129Z",
            "ConfigError": ""
        },
        "Ports": [
            {
                "IfName": "eth0",
                "Phylabel": "eth0",
                "Logicallabel": "eth0",
                "Alias": "",
                "IsMgmt": true,
                "IsL3Port": true,
                "Cost": 0,
                "Dhcp": 4,
                "Type": 0,
                "Subnet": {
                    "IP": "192.168.1.0",
                    "Mask": "////AA=="
                },
                "NtpServer": "",
                "DomainName": "",
                "DNSServers": [
                    "192.168.1.3"
                ],
                "NtpServers": null,
                "AddrInfoList": [
                    {
                        "Addr": "192.168.1.10",
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
                        "Addr": "fec0::6e70:6edc:1bd3:4119",
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
                        "Addr": "fe80::ac64:4852:7b50:dc95",
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
                "MacAddr": "UlQAEjRW",
                "DefaultRouters": [
                    "192.168.1.2",
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
                "LastFailed": "0001-01-01T00:00:00Z",
                "LastSucceeded": "2024-07-22T06:27:18.593306585Z",
                "LastError": ""
            },
            {
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
            }
        ]
    }
    "#;

    let result: DeviceNetworkStatus = serde_json::from_str(json_data).unwrap();
    assert_eq!(result.dpc_key, "lastresort");
    assert_eq!(result.state, DPCState::Success);
    assert_eq!(result.testing, false);
    assert_eq!(result.ports.is_some(), true);

    let ports = result.ports.unwrap();

    assert_eq!(ports.len(), 2);
    assert_eq!(ports[0].if_name, "eth0");
    assert_eq!(ports[1].if_name, "eth1");
}

#[test]
fn test_dpc_list_full() {
    let json_data = r#"
    {
        "CurrentIndex": 0,
        "PortConfigList": [
            {
                "Version": 1,
                "Key": "lastresort",
                "TimePriority": "1970-01-01T00:00:00Z",
                "State": 3,
                "ShaFile": "",
                "ShaValue": null,
                "LastFailed": "0001-01-01T00:00:00Z",
                "LastSucceeded": "2024-07-22T06:27:18.593415043Z",
                "LastError": "",
                "LastIPAndDNS": "2024-07-22T06:27:18.593414631Z",
                "Ports": [
                    {
                        "IfName": "eth0",
                        "USBAddr": "",
                        "PCIAddr": "",
                        "Phylabel": "eth0",
                        "Logicallabel": "eth0",
                        "Alias": "",
                        "NetworkUUID": "00000000-0000-0000-0000-000000000000",
                        "IsMgmt": true,
                        "IsL3Port": true,
                        "Cost": 0,
                        "Dhcp": 4,
                        "AddrSubnet": "",
                        "Gateway": "",
                        "DomainName": "",
                        "NTPServer": "",
                        "DNSServers": null,
                        "Type": 0,
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
                        "LastFailed": "0001-01-01T00:00:00Z",
                        "LastSucceeded": "2024-07-22T06:27:18.593306585Z",
                        "LastError": ""
                    },
                    {
                        "IfName": "eth1",
                        "USBAddr": "",
                        "PCIAddr": "",
                        "Phylabel": "eth1",
                        "Logicallabel": "eth1",
                        "Alias": "",
                        "NetworkUUID": "00000000-0000-0000-0000-000000000000",
                        "IsMgmt": true,
                        "IsL3Port": true,
                        "Cost": 0,
                        "Dhcp": 4,
                        "AddrSubnet": "",
                        "Gateway": "",
                        "DomainName": "",
                        "NTPServer": "",
                        "DNSServers": null,
                        "Type": 0,
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
                        "LastFailed": "2024-07-22T06:27:12.052879635Z",
                        "LastSucceeded": "0001-01-01T00:00:00Z",
                        "LastError": "All attempts to connect to https://zedcloud.alpha.zededa.net/api/v2/edgedevice/ping failed: interface eth1: no DNS server available"
                    }
                ]
            }
        ]
    }"#;
    let result: DevicePortConfigList = serde_json::from_str(json_data).unwrap();
    assert_eq!(result.current_index, 0);

    let port_list = result.port_config_list.unwrap();

    assert_eq!(port_list.len(), 1);
    assert_eq!(port_list[0].version, 1);
}

#[test]
fn dpc_list_null() {
    let json_data = r#"
    {
        "CurrentIndex": -1,
        "PortConfigList": null
    }"#;
    let result: DevicePortConfigList = serde_json::from_str(json_data).unwrap();
    assert_eq!(result.current_index, -1);
    assert_eq!(result.port_config_list, None);
}

#[test]
fn test_device_network_status_null_ports() {
    let json_data = r#"
    {
        "DPCKey": "",
        "Version": 0,
        "Testing": false,
        "State": 0,
        "CurrentIndex": 0,
        "RadioSilence": {
            "Imposed": false,
            "ChangeInProgress": false,
            "ChangeRequestedAt": "0001-01-01T00:00:00Z",
            "ConfigError": ""
        },
        "Ports": null
    }"#;

    let result: DeviceNetworkStatus = serde_json::from_str(json_data).unwrap();
    assert_eq!(result.dpc_key, "");
    assert_eq!(result.state, DPCState::None);
    assert_eq!(result.testing, false);
    assert_eq!(result.ports.is_none(), true);
}

#[derive(Deserialize, PartialEq, Debug)]
struct MacContainer {
    #[serde(deserialize_with = "deserialize_mac")]
    mac: MacAddr,
}

#[test]
fn test_deserialize_mac_addr() {
    let json = r#"{"mac":"UlQAEjRW"}"#;
    let mac_addr: MacContainer = serde_json::from_str(json).unwrap();
    assert_eq!(
        mac_addr,
        MacContainer {
            mac: MacAddr::from_str("52:54:00:12:34:56").unwrap()
        }
    );
}

#[test]
fn test_deserialize_mac_addr_invalid_length() {
    // 7 bytes instead of 6
    let json = r#"{"mac":"UlQAEjRXIg=="}"#;
    let mac_addr: Result<MacContainer, _> = serde_json::from_str(json);
    println!("{:?}", mac_addr);
    assert!(mac_addr
        .err()
        .unwrap()
        .to_string()
        .contains("invalid MAC address length"));
}

#[test]
fn test_deserialize_mac_addr_invalid_base64() {
    // Invalid base64 sequence
    let json = r#"{"mac":"UlQAE"}"#;
    let mac_addr: Result<MacContainer, _> = serde_json::from_str(json);
    assert!(mac_addr
        .err()
        .unwrap()
        .to_string()
        .contains("Invalid input length:"));
}
