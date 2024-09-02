use std::net::IpAddr;
use std::str::FromStr;

use super::*;
use async_inotify::app::parse;
use chrono::DateTime;
use chrono::Utc;
use eve_types::deserialize_mac;
use eve_types::AppInstanceStatus;
use eve_types::BondConfig;
use eve_types::BondMode;
use eve_types::DPCState;
use eve_types::DataSecAtRestStatus;
use eve_types::DeviceNetworkStatus;
use eve_types::DevicePortConfigList;
use eve_types::DhcpType;
use eve_types::ErrorSeverity;
use eve_types::GoIpNetwork;
use eve_types::LacpRate;
use eve_types::MIIMonitor;
use eve_types::NetworkPortStatus;
use eve_types::NetworkType;
use eve_types::RadioSilence;
use eve_types::ResultData;
use eve_types::WirelessCfg;
use eve_types::WirelessType;
use macaddr::MacAddr;
use message::IpcMessage;
use serde::Deserialize;
use serde_json::from_value;
use serde_json::json;

// Common conciderations for the tests:
// 1. Date and Time
// DateTime<UTC> and go date.Date are a bit different when it comes to serialization to a string
// in go trailing zeros are omited. HEre is the snippet from the go code:
// ----------------------------------------------------------------------
// fmtFrac formats the fraction of v/10**prec (e.g., ".12345") into the
// tail of buf, omitting trailing zeros. It omits the decimal
// point too when the fraction is 0. It returns the index where the
// output bytes begin and the value v/10**prec.
// ----------------------------------------------------------------------
// so some test data is fixed by hand to add the trailing zeros

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
                        "SharedLabels": null,
                        "Alias": "",
                        "NetworkUUID": "00000000-0000-0000-0000-000000000000",
                        "IsMgmt": true,
                        "IsL3Port": true,
                        "InvalidConfig": false,
                        "Cost": 0,
                        "MTU": 1500,
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
                        "SharedLabels": null,
                        "Alias": "",
                        "NetworkUUID": "00000000-0000-0000-0000-000000000000",
                        "IsMgmt": true,
                        "IsL3Port": true,
                        "InvalidConfig": false,
                        "Cost": 0,
                        "MTU": 1500,
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
    let dpc_list: DevicePortConfigList = serde_json::from_str(json_data).unwrap();

    // serialize back to json
    let new_json = serde_json::to_string_pretty(&dpc_list).unwrap();

    assert_eq!(
        json_data.parse::<serde_json::Value>().unwrap(),
        new_json.parse::<serde_json::Value>().unwrap()
    );

    assert_eq!(dpc_list.current_index, 0);

    let port_list = dpc_list.port_config_list.unwrap();

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
                        "LastIPAndDNS": "2024-07-28T19:14:16.557806570Z",
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
                                "SharedLabels": null,
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
                                "SharedLabels": null,
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
    let dpc_list: IpcMessage = serde_json::from_str(json_data).unwrap();

    // serialize back to json
    let new_json = serde_json::to_string_pretty(&dpc_list).unwrap();

    assert_eq!(
        json_data.parse::<serde_json::Value>().unwrap(),
        new_json.parse::<serde_json::Value>().unwrap()
    );

    match dpc_list {
        IpcMessage::DPCList(dpc_list) => {
            assert_eq!(dpc_list.current_index, 0);
            let dpc = dpc_list.get_dpc_by_key("lastresort").unwrap();
            let port = dpc.get_port_by_name("eth0").unwrap();
            assert_eq!(port.if_name, "eth0");
            assert_eq!(port.usb_addr, "");
        }
        _ => panic!("Expected IpcMessage::DPCList"),
    }
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

#[test]
fn test_io_adapters_1() {
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

#[test]
fn test_dpc_with_wifi_cypher() {
    let json_data = r#"
        {
            "type": "DPCList",
            "message": {
                "CurrentIndex": 0,
                "PortConfigList": [
                    {
                        "Version": 1,
                        "Key": "zedagent",
                        "TimePriority": "2024-08-05T21:37:05.087642277Z",
                        "State": 8,
                        "ShaFile": "",
                        "ShaValue": null,
                        "LastFailed": "2024-08-05T21:37:16.857215018Z",
                        "LastSucceeded": "0001-01-01T00:00:00Z",
                        "LastError": "physicalIO USB (phyLabel USB) is not a network adapter",
                        "LastIPAndDNS": "0001-01-01T00:00:00Z",
                        "Ports": [
                            {
                                "IfName": "eth0",
                                "USBAddr": "",
                                "PCIAddr": "0000:01:00.0",
                                "Phylabel": "eth0",
                                "Logicallabel": "eth0",
                                "SharedLabels": [
                                    "all",
                                    "uplink",
                                    "freeuplink"
                                ],
                                "Alias": "",
                                "NetworkUUID": "2e6038c1-ece6-4ffd-b95b-a7302c219d59",
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
                                "Type": 4,
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
                                "LastSucceeded": "0001-01-01T00:00:00Z",
                                "LastError": ""
                            },
                            {
                                "IfName": "eth1",
                                "USBAddr": "",
                                "PCIAddr": "0000:03:00.0",
                                "Phylabel": "eth1",
                                "Logicallabel": "eth1",
                                "SharedLabels": [
                                    "all",
                                    "uplink",
                                    "freeuplink"
                                ],
                                "Alias": "",
                                "NetworkUUID": "2e6038c1-ece6-4ffd-b95b-a7302c219d59",
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
                                "Type": 4,
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
                                "LastSucceeded": "0001-01-01T00:00:00Z",
                                "LastError": ""
                            },
                            {
                                "IfName": "wlan0",
                                "USBAddr": "",
                                "PCIAddr": "0000:02:00.0",
                                "Phylabel": "wlan0",
                                "Logicallabel": "wlan0",
                                "SharedLabels": [
                                    "all",
                                    "uplink",
                                    "freeuplink"
                                ],
                                "Alias": "",
                                "NetworkUUID": "f309d034-68d0-435f-986d-a8f12d9b6006",
                                "IsMgmt": true,
                                "IsL3Port": true,
                                "InvalidConfig": false,
                                "Cost": 0,
                                "MTU": 0,
                                "Dhcp": 4,
                                "AddrSubnet": "",
                                "Gateway": "",
                                "DomainName": "",
                                "NTPServer": "192.168.7.1",
                                "DNSServers": null,
                                "Type": 4,
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
                                    "WType": 2,
                                    "CellularV2": {
                                        "AccessPoints": null,
                                        "Probe": {
                                            "Disable": false,
                                            "Address": ""
                                        },
                                        "LocationTracking": false
                                    },
                                    "Wifi": [
                                        {
                                            "SSID": "Milan",
                                            "KeyScheme": 1,
                                            "Identity": "",
                                            "Password": "",
                                            "Priority": 0,
                                            "CipherBlockID": "f309d034-68d0-435f-986d-a8f12d9b6006-Milan",
                                            "CipherContextID": "3d829ff7-32f0-5295-84bd-a229a5dbedf6",
                                            "InitialValue": "WdRHlp5ePd3P1ucNJIMtvA==",
                                            "pubsub-large-CipherData": "+I1MISaSkjcFfkZ+Lq7nScTm7FYHc9DFCJdds9oWYjd7IRYUCWgGVLNUPYjgGpw/jfv8RiQDcPFTro+UNBPzUNLE",
                                            "ClearTextHash": "R5pBGu15+9xY5MKGiTZG55lYHGE5O/IVwZzNmEIgNis=",
                                            "IsCipher": true,
                                            "CipherContext": {
                                                "ContextID": "3d829ff7-32f0-5295-84bd-a229a5dbedf6",
                                                "HashScheme": 1,
                                                "KeyExchangeScheme": 1,
                                                "EncryptionScheme": 1,
                                                "ControllerCertHash": "uvaLT0Er26gi7u7chv4a3A==",
                                                "DeviceCertHash": "uakirbFVql9TY6nIxwRpog==",
                                                "Error": "",
                                                "ErrorTime": "0001-01-01T00:00:00Z",
                                                "ErrorSeverity": 0,
                                                "ErrorRetryCondition": "",
                                                "ErrorEntities": null
                                            },
                                            "Error": "",
                                            "ErrorTime": "0001-01-01T00:00:00Z",
                                            "ErrorSeverity": 0,
                                            "ErrorRetryCondition": "",
                                            "ErrorEntities": null
                                        }
                                    ],
                                    "Cellular": null
                                },
                                "LastFailed": "0001-01-01T00:00:00Z",
                                "LastSucceeded": "0001-01-01T00:00:00Z",
                                "LastError": ""
                            }
                        ]
                    },
                    {
                        "Version": 1,
                        "Key": "zedagent",
                        "TimePriority": "2024-08-05T21:36:55.720354176Z",
                        "State": 3,
                        "ShaFile": "",
                        "ShaValue": null,
                        "LastFailed": "2024-08-05T21:36:56.528687096Z",
                        "LastSucceeded": "2024-08-05T21:41:59.072285605Z",
                        "LastError": "",
                        "LastIPAndDNS": "2024-08-05T21:41:59.072285322Z",
                        "Ports": [
                            {
                                "IfName": "eth0",
                                "USBAddr": "",
                                "PCIAddr": "0000:01:00.0",
                                "Phylabel": "eth0",
                                "Logicallabel": "eth0",
                                "SharedLabels": [
                                    "all",
                                    "uplink",
                                    "freeuplink"
                                ],
                                "Alias": "",
                                "NetworkUUID": "2e6038c1-ece6-4ffd-b95b-a7302c219d59",
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
                                "Type": 4,
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
                                "LastSucceeded": "0001-01-01T00:00:00Z",
                                "LastError": ""
                            },
                            {
                                "IfName": "eth1",
                                "USBAddr": "",
                                "PCIAddr": "0000:03:00.0",
                                "Phylabel": "eth1",
                                "Logicallabel": "eth1",
                                "SharedLabels": [
                                    "all",
                                    "uplink",
                                    "freeuplink"
                                ],
                                "Alias": "",
                                "NetworkUUID": "2e6038c1-ece6-4ffd-b95b-a7302c219d59",
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
                                "Type": 4,
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
                                "LastSucceeded": "0001-01-01T00:00:00Z",
                                "LastError": ""
                            },
                            {
                                "IfName": "wlan0",
                                "USBAddr": "",
                                "PCIAddr": "0000:02:00.0",
                                "Phylabel": "wlan0",
                                "Logicallabel": "wlan0",
                                "SharedLabels": [
                                    "all",
                                    "uplink",
                                    "freeuplink"
                                ],
                                "Alias": "",
                                "NetworkUUID": "f309d034-68d0-435f-986d-a8f12d9b6006",
                                "IsMgmt": true,
                                "IsL3Port": true,
                                "InvalidConfig": false,
                                "Cost": 0,
                                "MTU": 0,
                                "Dhcp": 4,
                                "AddrSubnet": "",
                                "Gateway": "",
                                "DomainName": "",
                                "NTPServer": "192.168.7.1",
                                "DNSServers": null,
                                "Type": 4,
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
                                    "WType": 2,
                                    "CellularV2": {
                                        "AccessPoints": null,
                                        "Probe": {
                                            "Disable": false,
                                            "Address": ""
                                        },
                                        "LocationTracking": false
                                    },
                                    "Wifi": [
                                        {
                                            "SSID": "Milan",
                                            "KeyScheme": 1,
                                            "Identity": "",
                                            "Password": "",
                                            "Priority": 0,
                                            "CipherBlockID": "f309d034-68d0-435f-986d-a8f12d9b6006-Milan",
                                            "CipherContextID": "",
                                            "InitialValue": null,
                                            "pubsub-large-CipherData": null,
                                            "ClearTextHash": null,
                                            "IsCipher": false,
                                            "CipherContext": null,
                                            "Error": "",
                                            "ErrorTime": "0001-01-01T00:00:00Z",
                                            "ErrorSeverity": 0,
                                            "ErrorRetryCondition": "",
                                            "ErrorEntities": null
                                        }
                                    ],
                                    "Cellular": null
                                },
                                "LastFailed": "0001-01-01T00:00:00Z",
                                "LastSucceeded": "0001-01-01T00:00:00Z",
                                "LastError": ""
                            }
                        ]
                    }
                ]
            }
        }

        "#;
    let response: IpcMessage = serde_json::from_str(&json_data).unwrap();
}

#[test]
fn test_network_port_status_dns() {
    let json_data = r#"
        {
            "IfName": "eth0",
            "Phylabel": "eth0",
            "Logicallabel": "eth0",
            "SharedLabels": [
                "all",
                "uplink",
                "freeuplink"
            ],
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
                    "Addr": "fe80::b4f9:9a:708f:3b9b",
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
            "LastSucceeded": "2024-08-05T23:42:37.872333051Z",
            "LastError": ""
        }
    "#;
    let n: NetworkPortStatus = serde_json::from_str(&json_data).unwrap();
    assert_eq!(n.is_mgmt, true);
    assert_eq!(n.cost, 0);
    assert_eq!(n.dhcp, DhcpType::Client);
    assert_eq!(n.network_type, NetworkType::IPv4);
    assert_eq!(
        n.dns_servers,
        Some(vec!["10.208.13.254".parse::<IpAddr>().unwrap()])
    );
}

#[test]
fn test_app_status_full() {
    let json_data = r#"
        {
            "type": "AppStatus",
            "message": {
                "UUIDandVersion": {
                    "UUID": "7b1c286f-92ba-471c-916c-354817e1f970",
                    "Version": "1"
                },
                "DisplayName": "cs_nginx-1",
                "DomainName": "",
                "Activated": false,
                "ActivateInprogress": false,
                "FixedResources": {
                    "Kernel": "",
                    "Ramdisk": "",
                    "Memory": 524288,
                    "MaxMem": 524288,
                    "VCpus": 1,
                    "MaxCpus": 0,
                    "RootDev": "/dev/xvda1",
                    "ExtraArgs": "",
                    "BootLoader": "/usr/bin/pygrub",
                    "CPUs": "",
                    "DeviceTree": "",
                    "DtDev": null,
                    "IRQs": null,
                    "IOMem": null,
                    "VirtualizationMode": 0,
                    "EnableVnc": true,
                    "VncDisplay": 0,
                    "VncPasswd": "",
                    "CPUsPinned": false,
                    "VMMMaxMem": 0,
                    "EnableVncShimVM": false
                },
                "VolumeRefStatusList": [
                    {
                        "VolumeID": "8ac426df-38d6-4f7e-8a6d-723a1a9db644",
                        "GenerationCounter": 0,
                        "LocalGenerationCounter": 0,
                        "AppUUID": "7b1c286f-92ba-471c-916c-354817e1f970",
                        "State": 100,
                        "ActiveFileLocation": "",
                        "ContentFormat": 0,
                        "ReadOnly": false,
                        "DisplayName": "",
                        "MaxVolSize": 0,
                        "PendingAdd": false,
                        "WWN": "",
                        "VerifyOnly": true,
                        "Target": 0,
                        "CustomMeta": "",
                        "ReferenceName": "",
                        "ErrorSourceType": "",
                        "Error": "",
                        "ErrorTime": "0001-01-01T00:00:00Z",
                        "ErrorSeverity": 0,
                        "ErrorRetryCondition": "",
                        "ErrorEntities": null
                    }
                ],
                "AppNetAdapters": null,
                "BootTime": "0001-01-01T00:00:00Z",
                "IoAdapterList": null,
                "RestartInprogress": 0,
                "RestartStartedAt": "0001-01-01T00:00:00Z",
                "PurgeInprogress": 0,
                "PurgeStartedAt": "0001-01-01T00:00:00Z",
                "State": 100,
                "MissingNetwork": false,
                "MissingMemory": false,
                "ErrorSourceType": "",
                "Error": "",
                "ErrorTime": "0001-01-01T00:00:00Z",
                "ErrorSeverity": 0,
                "ErrorRetryCondition": "",
                "ErrorEntities": null,
                "StartTime": "2024-08-06T11:51:10.714191287Z",
                "SnapStatus": {
                    "MaxSnapshots": 1,
                    "RequestedSnapshots": null,
                    "AvailableSnapshots": null,
                    "SnapshotsToBeDeleted": null,
                    "PreparedVolumesSnapshotConfigs": null,
                    "SnapshotOnUpgrade": false,
                    "HasRollbackRequest": false,
                    "ActiveSnapshot": "",
                    "RollbackInProgress": false
                },
                "MemOverhead": 0
            }
        }
        "#;
    let _: IpcMessage = serde_json::from_str(json_data).unwrap();
}

#[test]
fn test_onboarding_status() {
    let json_data = r#"
        {
          "type": "OnboardingStatus",
          "message": {
            "DeviceUUID": "584a643c-c8b1-45d5-a287-84fc81e0d39c",
            "HardwareModel": "QEMU.Standard PC (Q35 + ICH9, 2009)"
          }
        }
        "#;
    let _: IpcMessage = serde_json::from_str(json_data).unwrap();
}

#[test]
fn test_vault_status() {
    let json_data = r#"
        {
          "type": "VaultStatus",
          "message": {
            "Name": "Application Data Store",
            "Status": 1,
            "PCRStatus": 2,
            "ConversionComplete": true,
            "Error": "TPM is either absent or not in use",
            "ErrorTime": "2024-08-28T20:15:08.246947098Z",
            "ErrorSeverity": 3,
            "ErrorRetryCondition": "",
            "ErrorEntities": null
          }
        }
        "#;
    let _: IpcMessage = serde_json::from_str(json_data).unwrap();
}

#[test]
fn test_vault_status_error() {
    let json_data = r#"
        {
          "type": "VaultStatus",
          "message": {
            "Name": "Application Data Store",
            "Status": 4,
            "PCRStatus": 1,
            "ConversionComplete": false,
            "MissmatchingPCRs": [8, 13],
            "Error": "Vault key unavailable",
            "ErrorTime": "2024-08-31T18:54:42.220924775Z",
            "ErrorSeverity": 3,
            "ErrorRetryCondition": "",
            "ErrorEntities": null
          }
        }
        "#;
    let msg: IpcMessage = serde_json::from_str(json_data).unwrap();
    match msg {
        IpcMessage::VaultStatus(eve_status) => {
            assert_eq!(eve_status.status, DataSecAtRestStatus::DataSecAtRestError);
            assert_eq!(
                eve_status.error_and_time.error_description.error,
                "Vault key unavailable"
            );
            // assert_eq!(
            //     eve_status.error_and_time.error_description.error_time,
            //     DateTime::parse::<Utc>("2024-08-31T18:54:42.220924775Z").unwrap()
            // );
            assert_eq!(
                eve_status.error_and_time.error_description.error_severity,
                ErrorSeverity::Error
            );
        }
        _ => panic!("unexpected message type"),
    }
}

#[test]
fn test_app_summary() {
    let json_data = r#"
        {
          "type": "AppSummary",
          "message": {
            "UUIDandVersion": {
              "UUID": "00000000-0000-0000-0000-000000000000",
              "Version": ""
            },
            "TotalStarting": 1,
            "TotalRunning": 0,
            "TotalStopping": 0,
            "TotalError": 0
          }
        }
        "#;
    let _: IpcMessage = serde_json::from_str(json_data).unwrap();
}

#[test]
fn test_node_status_not_onboarded() {
    let json_data = r#"
        {
          "type": "NodeStatus",
          "message": {
            "server": "zedcloud.alpha.zededa.net",
            "node_uuid": "00000000-0000-0000-0000-000000000000",
            "onboarded": false
          }
        }
        "#;
    let msg: IpcMessage = serde_json::from_str(json_data).unwrap();
    match msg {
        IpcMessage::NodeStatus(message) => {
            assert_eq!(message.onboarded, false);
            assert_eq!(message.node_uuid, None);
            assert_eq!(
                message.server,
                Some("zedcloud.alpha.zededa.net".to_string())
            )
        }
        _ => panic!("unexpected message type"),
    }
}

#[test]
fn test_node_status_not_onboarded_emty_server() {
    let json_data = r#"
        {
          "type": "NodeStatus",
          "message": {
            "node_uuid": "00000000-0000-0000-0000-000000000000",
            "onboarded": false
          }
        }
        "#;
    let msg: IpcMessage = serde_json::from_str(json_data).unwrap();
    match msg {
        IpcMessage::NodeStatus(message) => {
            assert_eq!(message.onboarded, false);
            assert_eq!(message.node_uuid, None);
            assert_eq!(message.server, None)
        }
        _ => panic!("unexpected message type"),
    }
}

#[test]
fn test_app_status_with_error() {
    let json_data = r#"
        {
          "type": "AppStatus",
          "message": {
            "UUIDandVersion": {
              "UUID": "68cec03f-3c11-4dbe-8089-5d6f3dc3c872",
              "Version": "1"
            },
            "DisplayName": "cs_nginx-qemu-1",
            "DomainName": "",
            "Activated": false,
            "ActivateInprogress": false,
            "FixedResources": {
              "Kernel": "",
              "Ramdisk": "",
              "Memory": 524288,
              "MaxMem": 524288,
              "VCpus": 1,
              "MaxCpus": 0,
              "RootDev": "/dev/xvda1",
              "ExtraArgs": "",
              "BootLoader": "/usr/bin/pygrub",
              "CPUs": "",
              "DeviceTree": "",
              "DtDev": null,
              "IRQs": null,
              "IOMem": null,
              "VirtualizationMode": 0,
              "EnableVnc": true,
              "VncDisplay": 0,
              "VncPasswd": "",
              "CPUsPinned": false,
              "VMMMaxMem": 0,
              "EnableVncShimVM": false
            },
            "VolumeRefStatusList": [
              {
                "VolumeID": "245238bd-9cb6-4601-a77a-43d21a0f1c2f",
                "GenerationCounter": 0,
                "LocalGenerationCounter": 0,
                "AppUUID": "68cec03f-3c11-4dbe-8089-5d6f3dc3c872",
                "State": 101,
                "ActiveFileLocation": "",
                "ContentFormat": 0,
                "ReadOnly": false,
                "DisplayName": "cs_nginx-qemu-1_0_m_0",
                "MaxVolSize": 0,
                "PendingAdd": false,
                "WWN": "",
                "VerifyOnly": true,
                "Target": 0,
                "CustomMeta": "",
                "ReferenceName": "",
                "ErrorSourceType": "types.VolumeStatus",
                "Error": "Found error in content tree cs_nginx_image attached to volume cs_nginx-qemu-1_0_m_0: lookupDatastoreConfig(c1b0cb8a-9b39-4b9a-82ff-b5409757ecc2) error: Get(zedagent/DatastoreConfig) unknown key c1b0cb8a-9b39-4b9a-82ff-b5409757ecc2",
                "ErrorTime": "2024-08-31T10:17:58.660422709Z",
                "ErrorSeverity": 1,
                "ErrorRetryCondition": "Will retry when datastore available",
                "ErrorEntities": [
                  {
                    "EntityType": 11,
                    "EntityID": "245238bd-9cb6-4601-a77a-43d21a0f1c2f"
                  }
                ]
              }
            ],
            "AppNetAdapters": null,
            "BootTime": "0001-01-01T00:00:00Z",
            "IoAdapterList": null,
            "RestartInprogress": 0,
            "RestartStartedAt": "0001-01-01T00:00:00Z",
            "PurgeInprogress": 0,
            "PurgeStartedAt": "0001-01-01T00:00:00Z",
            "State": 101,
            "MissingNetwork": false,
            "MissingMemory": false,
            "ErrorSourceType": "types.VolumeStatus",
            "Error": "Found error in content tree cs_nginx_image attached to volume cs_nginx-qemu-1_0_m_0: lookupDatastoreConfig(c1b0cb8a-9b39-4b9a-82ff-b5409757ecc2) error: Get(zedagent/DatastoreConfig) unknown key c1b0cb8a-9b39-4b9a-82ff-b5409757ecc2\n\n",
            "ErrorTime": "2024-08-31T10:17:58.660422709Z",
            "ErrorSeverity": 1,
            "ErrorRetryCondition": "Will retry when datastore available",
            "ErrorEntities": [
              { "EntityType": 11, "EntityID": "245238bd-9cb6-4601-a77a-43d21a0f1c2f" }
            ],
            "StartTime": "2024-08-31T10:17:51.591626391Z",
            "SnapStatus": {
              "MaxSnapshots": 1,
              "RequestedSnapshots": null,
              "AvailableSnapshots": null,
              "SnapshotsToBeDeleted": null,
              "PreparedVolumesSnapshotConfigs": null,
              "SnapshotOnUpgrade": false,
              "HasRollbackRequest": false,
              "ActiveSnapshot": "",
              "RollbackInProgress": false
            },
            "MemOverhead": 0
          }
        }
        "#;
    let _msg: IpcMessage = serde_json::from_str(json_data).unwrap();
}
