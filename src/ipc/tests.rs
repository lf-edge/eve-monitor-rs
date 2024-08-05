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
use message::IpcMessage;
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
            mask: Some("////AA==".to_string())
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
    assert_eq!(result.mask, Some("////AA==".to_string()));
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
fn test_device_network_status_full_1() {
    let json_data = r#"
{
    "type": "NetworkStatus",
    "message": {
        "DPCKey": "zedagent",
        "Version": 1,
        "Testing": true,
        "State": 8,
        "CurrentIndex": 0,
        "RadioSilence": {
            "Imposed": false,
            "ChangeInProgress": false,
            "ChangeRequestedAt": "2024-07-28T19:13:56.913387681Z",
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
                "InvalidConfig": false,
                "Cost": 0,
                "Dhcp": 4,
                "Type": 4,
                "Subnet": {
                    "IP": "10.208.13.0",
                    "Mask": "////AA=="
                },
                "NtpServer": "",
                "DomainName": "",
                "DNSServers": [
                    "10.208.13.254"
                ],
                "NtpServers": [
                    "194.164.164.175",
                    "49.12.199.148"
                ],
                "AddrInfoList": [
                    {
                        "Addr": "10.208.13.81",
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
                        "Addr": "fe80::dce:80a7:3a27:cbc2",
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
                "MacAddr": "AOBLai/6",
                "DefaultRouters": [
                    "10.208.13.254"
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
                "LastSucceeded": "0001-01-01T00:00:00Z",
                "LastError": ""
            },
            {
                "IfName": "eth1",
                "Phylabel": "eth1",
                "Logicallabel": "eth1",
                "Alias": "",
                "IsMgmt": true,
                "IsL3Port": true,
                "InvalidConfig": false,
                "Cost": 0,
                "Dhcp": 4,
                "Type": 4,
                "Subnet": {
                    "IP": "10.208.13.0",
                    "Mask": "////AA=="
                },
                "NtpServer": "",
                "DomainName": "",
                "DNSServers": [
                    "10.208.13.254"
                ],
                "NtpServers": [
                    "194.164.164.175",
                    "49.12.199.148"
                ],
                "AddrInfoList": [
                    {
                        "Addr": "10.208.13.194",
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
                        "Addr": "fe80::b398:f42a:f1e5:a80f",
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
                "MacAddr": "AOBLYm25",
                "DefaultRouters": [
                    "10.208.13.254"
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
                "LastSucceeded": "0001-01-01T00:00:00Z",
                "LastError": ""
            },
            {
                "IfName": "wwan0",
                "Phylabel": "wwan0",
                "Logicallabel": "wwan0",
                "Alias": "",
                "IsMgmt": true,
                "IsL3Port": true,
                "InvalidConfig": false,
                "Cost": 1,
                "Dhcp": 4,
                "Type": 4,
                "Subnet": {
                    "IP": "",
                    "Mask": null
                },
                "NtpServer": "",
                "DomainName": "",
                "DNSServers": null,
                "NtpServers": null,
                "AddrInfoList": null,
                "Up": false,
                "MacAddr": "vs1R/K1h",
                "DefaultRouters": null,
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
                "LastSucceeded": "0001-01-01T00:00:00Z",
                "LastError": ""
            },
            {
                "IfName": "wwan1",
                "Phylabel": "wwan1",
                "Logicallabel": "wwan1",
                "Alias": "",
                "IsMgmt": true,
                "IsL3Port": true,
                "InvalidConfig": false,
                "Cost": 1,
                "Dhcp": 4,
                "Type": 4,
                "Subnet": {
                    "IP": "",
                    "Mask": "AAAAAA=="
                },
                "NtpServer": "",
                "DomainName": "",
                "DNSServers": null,
                "NtpServers": null,
                "AddrInfoList": null,
                "Up": false,
                "MacAddr": "shbfDs6F",
                "DefaultRouters": null,
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
                "LastSucceeded": "0001-01-01T00:00:00Z",
                "LastError": ""
            }
        ]
    }
}
    "#;
    let result: IpcMessage = serde_json::from_str(json_data).unwrap();
    // assert_eq!(result.dpc_key, "lastresort");
    // assert_eq!(result.state, DPCState::Success);
    // assert_eq!(result.testing, false);
    // assert_eq!(result.ports.is_some(), true);

    // let ports = result.ports.unwrap();

    // assert_eq!(ports.len(), 2);
    // assert_eq!(ports[0].if_name, "eth0");
    // assert_eq!(ports[1].if_name, "eth1");
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
fn test_dpc_list_full_1() {
    let json_data = r#"
    {
            "type": "DPCList",
            "message": {
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
                        "LastSucceeded": "2024-07-28T19:14:16.557806833Z",
                        "LastError": "",
                        "LastIPAndDNS": "2024-07-28T19:14:16.55780657Z",
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
                                "InvalidConfig": false,
                                "Cost": 0,
                                "MTU": 0,
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
                                "LastFailed": "2024-07-28T19:14:07.743485163Z",
                                "LastSucceeded": "2024-07-28T19:14:16.557756976Z",
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
                                "InvalidConfig": false,
                                "Cost": 0,
                                "MTU": 0,
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
                                "LastFailed": "2024-07-28T19:14:07.742642939Z",
                                "LastSucceeded": "0001-01-01T00:00:00Z",
                                "LastError": "All attempts to connect to https://zedcloud.alpha.zededa.net/api/v2/edgedevice/ping failed: interface eth1: no DNS server available"
                            }
                        ]
                    }
                ]
            }
    }
    "#;
    let result: IpcMessage = serde_json::from_str(json_data).unwrap();
    // assert_eq!(result.current_index, 0);

    // let port_list = result.port_config_list.unwrap();

    // assert_eq!(port_list.len(), 1);
    // assert_eq!(port_list[0].version, 1);
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

#[test]
fn test_response_ok_deserialization() {
    let json = r#"{"type":"Response","message":{"Ok":"ok","id":12}}"#;
    let response: IpcMessage = serde_json::from_str(json).unwrap();

    match response {
        IpcMessage::Response { result, id } => {
            assert_eq!(result, Ok("ok".to_string()));
            assert_eq!(id, 12);
        }
        _ => panic!("Unexpected message type"),
    }
}

#[test]
fn test_response_err_deserialization() {
    let json = r#"{"type":"Response","message":{"Err":"error","id":12}}"#;
    let response: IpcMessage = serde_json::from_str(json).unwrap();

    match response {
        IpcMessage::Response { result, id } => {
            assert_eq!(result, Err("error".to_string()));
            assert_eq!(id, 12);
        }
        _ => panic!("Unexpected message type"),
    }
}

#[test]
fn test_downloader_1() {
    let json_data = r#"
    {
        "type": "DownloaderStatus",
        "message": {
            "ImageSha256": "a72860cb95fd59e9c696c66441c64f18e66915fa26b249911e83c3854477ed9a",
            "DatastoreIDList": [
                "c1b0cb8a-9b39-4b9a-82ff-b5409757ecc2"
            ],
            "Target": "/persist/vault/downloader/pending/95ac1bd0a96cbd6099fd9e8521bffec5902b6e5c31542f564598392aa53233de.a72860cb95fd59e9c696c66441c64f18e66915fa26b249911e83c3854477ed9a",
            "Name": "nginx@sha256:a72860cb95fd59e9c696c66441c64f18e66915fa26b249911e83c3854477ed9a",
            "RefCount": 1,
            "LastUse": "2024-07-29T07:09:41.887178121Z",
            "Expired": false,
            "NameIsURL": false,
            "State": 104,
            "ReservedSpace": 0,
            "Size": 7296,
            "TotalSize": 7296,
            "CurrentSize": 7296,
            "Progress": 100,
            "ModTime": "2024-07-29T07:09:44.044481071Z",
            "ContentType": "",
            "Error": "",
            "ErrorTime": "0001-01-01T00:00:00Z",
            "ErrorSeverity": 0,
            "ErrorRetryCondition": "",
            "ErrorEntities": null,
            "RetryCount": 0,
            "OrigError": ""
        }
    }
    "#;
    let response: IpcMessage = serde_json::from_str(&json_data).unwrap();
}

#[test]
fn test_io_adapters() {
    let json_data = r#"
        {
            "type": "IOAdapters",
            "message": {
                "Initialized": true,
                "AdapterList": [
                    {
                        "Ptype": 4,
                        "Phylabel": "Audio",
                        "Phyaddr": {
                            "PciLong": "0000:00:0e.0",
                            "Ifname": "",
                            "Serial": "",
                            "Irq": "",
                            "Ioports": "",
                            "UsbAddr": "",
                            "UsbProduct": "",
                            "UnknownType": ""
                        },
                        "Logicallabel": "Audio",
                        "Assigngrp": "group2",
                        "Parentassigngrp": "",
                        "Usage": 0,
                        "UsagePolicy": {
                            "FreeUplink": false
                        },
                        "Vfs": {
                            "Count": 0,
                            "Data": null
                        },
                        "Cbattr": null
                    },
                    {
                        "Ptype": 3,
                        "Phylabel": "COM1",
                        "Phyaddr": {
                            "PciLong": "",
                            "Ifname": "",
                            "Serial": "/dev/ttyS2",
                            "Irq": "10",
                            "Ioports": "3e8-3ef",
                            "UsbAddr": "",
                            "UsbProduct": "",
                            "UnknownType": ""
                        },
                        "Logicallabel": "COM1",
                        "Assigngrp": "COM1",
                        "Parentassigngrp": "",
                        "Usage": 0,
                        "UsagePolicy": {
                            "FreeUplink": false
                        },
                        "Vfs": {
                            "Count": 0,
                            "Data": null
                        },
                        "Cbattr": null
                    },
                    {
                        "Ptype": 3,
                        "Phylabel": "RS485-1",
                        "Phyaddr": {
                            "PciLong": "",
                            "Ifname": "",
                            "Serial": "/dev/ttyXRUSB0",
                            "Irq": "",
                            "Ioports": "",
                            "UsbAddr": "",
                            "UsbProduct": "",
                            "UnknownType": ""
                        },
                        "Logicallabel": "RS485-1",
                        "Assigngrp": "RS485-1",
                        "Parentassigngrp": "",
                        "Usage": 0,
                        "UsagePolicy": {
                            "FreeUplink": false
                        },
                        "Vfs": {
                            "Count": 0,
                            "Data": null
                        },
                        "Cbattr": null
                    },
                    {
                        "Ptype": 3,
                        "Phylabel": "RS485-2",
                        "Phyaddr": {
                            "PciLong": "",
                            "Ifname": "",
                            "Serial": "/dev/ttyXRUSB1",
                            "Irq": "",
                            "Ioports": "",
                            "UsbAddr": "",
                            "UsbProduct": "",
                            "UnknownType": ""
                        },
                        "Logicallabel": "RS485-2",
                        "Assigngrp": "RS485-2",
                        "Parentassigngrp": "",
                        "Usage": 0,
                        "UsagePolicy": {
                            "FreeUplink": false
                        },
                        "Vfs": {
                            "Count": 0,
                            "Data": null
                        },
                        "Cbattr": null
                    },
                    {
                        "Ptype": 3,
                        "Phylabel": "RS485-3",
                        "Phyaddr": {
                            "PciLong": "",
                            "Ifname": "",
                            "Serial": "/dev/ttyXRUSB2",
                            "Irq": "",
                            "Ioports": "",
                            "UsbAddr": "",
                            "UsbProduct": "",
                            "UnknownType": ""
                        },
                        "Logicallabel": "RS485-3",
                        "Assigngrp": "RS485-3",
                        "Parentassigngrp": "",
                        "Usage": 0,
                        "UsagePolicy": {
                            "FreeUplink": false
                        },
                        "Vfs": {
                            "Count": 0,
                            "Data": null
                        },
                        "Cbattr": null
                    },
                    {
                        "Ptype": 3,
                        "Phylabel": "RS485-4",
                        "Phyaddr": {
                            "PciLong": "",
                            "Ifname": "",
                            "Serial": "/dev/ttyXRUSB3",
                            "Irq": "",
                            "Ioports": "",
                            "UsbAddr": "",
                            "UsbProduct": "",
                            "UnknownType": ""
                        },
                        "Logicallabel": "RS485-4",
                        "Assigngrp": "RS485-4",
                        "Parentassigngrp": "",
                        "Usage": 0,
                        "UsagePolicy": {
                            "FreeUplink": false
                        },
                        "Vfs": {
                            "Count": 0,
                            "Data": null
                        },
                        "Cbattr": null
                    },
                    {
                        "Ptype": 13,
                        "Phylabel": "USB",
                        "Phyaddr": {
                            "PciLong": "0000:00:15.0",
                            "Ifname": "",
                            "Serial": "",
                            "Irq": "",
                            "Ioports": "",
                            "UsbAddr": "",
                            "UsbProduct": "",
                            "UnknownType": ""
                        },
                        "Logicallabel": "USB",
                        "Assigngrp": "group8",
                        "Parentassigngrp": "",
                        "Usage": 0,
                        "UsagePolicy": {
                            "FreeUplink": false
                        },
                        "Vfs": {
                            "Count": 0,
                            "Data": null
                        },
                        "Cbattr": null
                    },
                    {
                        "Ptype": 7,
                        "Phylabel": "VGA",
                        "Phyaddr": {
                            "PciLong": "0000:00:02.0",
                            "Ifname": "",
                            "Serial": "",
                            "Irq": "",
                            "Ioports": "",
                            "UsbAddr": "",
                            "UsbProduct": "",
                            "UnknownType": ""
                        },
                        "Logicallabel": "VGA",
                        "Assigngrp": "group0",
                        "Parentassigngrp": "",
                        "Usage": 0,
                        "UsagePolicy": {
                            "FreeUplink": false
                        },
                        "Vfs": {
                            "Count": 0,
                            "Data": null
                        },
                        "Cbattr": null
                    },
                    {
                        "Ptype": 1,
                        "Phylabel": "eth0",
                        "Phyaddr": {
                            "PciLong": "0000:01:00.0",
                            "Ifname": "eth0",
                            "Serial": "",
                            "Irq": "",
                            "Ioports": "",
                            "UsbAddr": "",
                            "UsbProduct": "",
                            "UnknownType": ""
                        },
                        "Logicallabel": "eth0",
                        "Assigngrp": "group14",
                        "Parentassigngrp": "",
                        "Usage": 1,
                        "UsagePolicy": {
                            "FreeUplink": false
                        },
                        "Vfs": {
                            "Count": 0,
                            "Data": null
                        },
                        "Cbattr": null
                    },
                    {
                        "Ptype": 1,
                        "Phylabel": "eth1",
                        "Phyaddr": {
                            "PciLong": "0000:03:00.0",
                            "Ifname": "eth1",
                            "Serial": "",
                            "Irq": "",
                            "Ioports": "",
                            "UsbAddr": "",
                            "UsbProduct": "",
                            "UnknownType": ""
                        },
                        "Logicallabel": "eth1",
                        "Assigngrp": "group16",
                        "Parentassigngrp": "",
                        "Usage": 1,
                        "UsagePolicy": {
                            "FreeUplink": false
                        },
                        "Vfs": {
                            "Count": 0,
                            "Data": null
                        },
                        "Cbattr": null
                    },
                    {
                        "Ptype": 5,
                        "Phylabel": "wlan0",
                        "Phyaddr": {
                            "PciLong": "0000:02:00.0",
                            "Ifname": "wlan0",
                            "Serial": "",
                            "Irq": "",
                            "Ioports": "",
                            "UsbAddr": "",
                            "UsbProduct": "",
                            "UnknownType": ""
                        },
                        "Logicallabel": "wlan0",
                        "Assigngrp": "group15",
                        "Parentassigngrp": "",
                        "Usage": 1,
                        "UsagePolicy": {
                            "FreeUplink": false
                        },
                        "Vfs": {
                            "Count": 0,
                            "Data": null
                        },
                        "Cbattr": null
                    }
                ]
            }
        }
        "#;
    let response: IpcMessage = serde_json::from_str(&json_data).unwrap();
}
