# CC2 MQTT API Documentation - Centauri Carbon 2

The Centauri Carbon 2 (CC2) uses a fundamentally different communication protocol than the CC1. Instead of the SDCP WebSocket protocol, CC2 uses an **inverted MQTT architecture** where the printer itself runs the MQTT broker.

**Source**: Derived from analysis of the [elegoo-link](https://github.com/ELEGOO-3D/elegoo-link) open-source library and community reverse engineering efforts.

!!! info "Key Difference from CC1"
    Unlike CC1 where the printer is a client connecting to a WebSocket server, the CC2 printer **IS the server** - it runs an MQTT broker that clients connect to.

## Protocol Overview

| Feature | Description |
|---------|-------------|
| Transport | MQTT 3.1.1 over TCP |
| Broker | Runs on the printer (port 1883) |
| Discovery | UDP broadcast (port 52700) |
| Authentication | Username/password + optional access code |
| Status Updates | Delta-based (incremental) |
| Max Clients | ~4 concurrent connections |
| Heartbeat | Required every 10 seconds |

### Supported Printers

The CC2 protocol is used by:

- Elegoo Centauri Carbon 2
- Elegoo Cura (some models)
- Other Elegoo FDM printers with CC2 firmware

---

## Network Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Local Network                                  │
│                                                                          │
│   ┌─────────────────┐              ┌──────────────────────────────────┐ │
│   │   Client App    │              │       CC2 Printer                │ │
│   │   (Slicer,      │              │                                  │ │
│   │   Home Asst.)   │              │  ┌─────────────────────────────┐ │ │
│   │                 │   UDP:52700  │  │   Discovery Service         │ │ │
│   │  ┌───────────┐  │◄────────────►│  │   (responds to broadcasts)  │ │ │
│   │  │ Discovery │  │              │  └─────────────────────────────┘ │ │
│   │  └───────────┘  │              │                                  │ │
│   │                 │              │  ┌─────────────────────────────┐ │ │
│   │  ┌───────────┐  │  TCP:1883    │  │   MQTT Broker               │ │ │
│   │  │ MQTT      │  │◄────────────►│  │   (handles subscriptions,   │ │ │
│   │  │ Client    │  │              │  │    publishes status)        │ │ │
│   │  └───────────┘  │              │  └─────────────────────────────┘ │ │
│   │                 │              │                                  │ │
│   │  ┌───────────┐  │  TCP:8080    │  ┌─────────────────────────────┐ │ │
│   │  │ HTTP      │  │◄────────────►│  │   HTTP Server               │ │ │
│   │  │ Client    │  │              │  │   (file upload, video)      │ │ │
│   │  └───────────┘  │              │  └─────────────────────────────┘ │ │
│   │                 │              │                                  │ │
│   └─────────────────┘              └──────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Ports Used

| Port | Protocol | Direction | Purpose |
|------|----------|-----------|---------|
| 52700 | UDP | Client → Printer (broadcast) | Discovery |
| 1883 | TCP | Client → Printer | MQTT commands/status |
| 8080 | TCP | Client → Printer | HTTP (video, file transfers) |

---

## Connection Establishment

### Connection Flow

```
┌─────────┐   ┌───────────┐   ┌──────────┐   ┌─────────┐   ┌────────┐
│Discovery│──►│MQTT       │──►│Register  │──►│Subscribe│──►│Heartbeat│
│(UDP)    │   │Connect    │   │          │   │Topics   │   │Loop     │
└─────────┘   └───────────┘   └──────────┘   └─────────┘   └────────┘
```

---

## Discovery Protocol

Discovery allows clients to find CC2 printers on the local network.

### Configuration

| Parameter | Value |
|-----------|-------|
| Port | 52700 (UDP) |
| Method | Broadcast or directed UDP |
| Timeout | 5 seconds recommended |

### Discovery Request

Send to `255.255.255.255:52700` (broadcast) or `<printer_ip>:52700` (direct):

```json
{
  "id": 0,
  "method": 7000
}
```

### Discovery Response

```json
{
  "id": 0,
  "result": {
    "host_name": "Centauri Carbon 2",
    "machine_model": "Centauri Carbon 2",
    "sn": "CC2ABCD1234567890",
    "token_status": 0,
    "lan_status": 1
  }
}
```

### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `host_name` | string | User-configured printer name (can be changed in settings) |
| `machine_model` | string | Hardware model identifier |
| `sn` | string | Serial number (unique, used in MQTT topics) |
| `token_status` | int | Authentication mode (see below) |
| `lan_status` | int | Network mode (see below) |

### Token Status Values

| Value | Meaning | Action Required |
|-------|---------|-----------------|
| 0 | No access code | Use default password `123456` |
| 1 | Access code set | Use access code as MQTT password |

### LAN Status Values

| Value | Meaning |
|-------|---------|
| 0 | Cloud mode (printer may be cloud-connected) |
| 1 | LAN-only mode |

### Python Example

```python
import socket
import json

# Send UDP discovery broadcast
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
sock.settimeout(5.0)

message = json.dumps({"id": 0, "method": 7000}).encode()
sock.sendto(message, ('255.255.255.255', 52700))

# Receive response
data, addr = sock.recvfrom(1024)
response = json.loads(data)
printer_ip = addr[0]
serial_number = response['result']['sn']
print(f"Found printer at {printer_ip}: {serial_number}")
```

!!! note
    The printer may take 1-2 seconds to respond. Multiple printers on the network will all respond. Discovery works even when another client is connected.

---

## MQTT Connection

After discovering the printer, establish an MQTT connection.

### Connection Parameters

| Parameter | Value |
|-----------|-------|
| Host | Printer IP address |
| Port | 1883 |
| Protocol | MQTT 3.1.1 |
| Keep-alive | 60 seconds |
| Clean Session | true |
| Username | `elegoo` |
| Password | `123456` or access code (if `token_status=1`) |

### Client ID Format

Generate a unique client ID:

```
1_PC_<random 4 digits>
```

Examples: `1_PC_4521`, `1_PC_8734`, `1_PC_0001`

The format is: `<platform>_<device_type>_<random>`

- Platform: `1` (appears constant)
- Device type: `PC`, `APP`, etc.
- Random: 4 digits for uniqueness

### Request ID Format

Used for registration:

```
<client_id>_req
```

Example: `1_PC_4521_req`

### Connection Example

```python
import paho.mqtt.client as mqtt
import random

client_id = f"1_PC_{random.randint(1000, 9999)}"
request_id = f"{client_id}_req"

client = mqtt.Client(client_id=client_id)
client.username_pw_set("elegoo", "123456")  # or access code
client.connect(printer_ip, 1883, keepalive=60)
```

### Connection Error Handling

| Error | Cause | Solution |
|-------|-------|----------|
| Connection refused | Wrong IP/port | Verify discovery response |
| Authentication failed | Wrong password | Check `token_status`, use access code |
| Connection closed | Too many clients | Wait for slot, implement reconnection |
| Timeout | Network issue | Retry with backoff |

---

## Registration Protocol

!!! warning "Registration Required"
    Registration **must** be completed before sending any commands. The printer needs to know which client IDs are valid.

### Registration Flow

```
┌────────┐                              ┌────────────┐
│ Client │                              │  Printer   │
└───┬────┘                              └─────┬──────┘
    │                                         │
    │ 1. Subscribe to register_response       │
    │────────────────────────────────────────►│
    │                                         │
    │ 2. Publish registration request         │
    │────────────────────────────────────────►│
    │                                         │
    │ 3. Receive registration response        │
    │◄────────────────────────────────────────│
    │                                         │
    │ 4. If "ok", proceed with commands       │
    │                                         │
```

### Topics

| Action | Topic |
|--------|-------|
| Subscribe (response) | `elegoo/<sn>/<request_id>/register_response` |
| Publish (request) | `elegoo/<sn>/api_register` |

### Registration Request

```json
{
  "client_id": "1_PC_4521",
  "request_id": "1_PC_4521_req"
}
```

### Registration Response - Success

```json
{
  "client_id": "1_PC_4521",
  "error": "ok"
}
```

### Registration Response - Failure

```json
{
  "client_id": "1_PC_4521",
  "error": "too many clients"
}
```

### Error Values

| Error | Meaning | Action |
|-------|---------|--------|
| `ok` | Success | Proceed with subscriptions |
| `fail` | General failure | Retry after delay |
| `too many clients` | Max connections reached | Wait or disconnect another client |

### Timing

| Parameter | Value |
|-----------|-------|
| Timeout | 3 seconds |
| Max Clients | ~4 (may vary by firmware) |
| Retry Delay | 5-10 seconds recommended |

---

## Topic Structure

After registration, subscribe to these topics:

| Purpose | Topic |
|---------|-------|
| Status updates | `elegoo/<sn>/api_status` |
| Command responses | `elegoo/<sn>/<client_id>/api_response` |
| Send commands | `elegoo/<sn>/<client_id>/api_request` |

---

## Heartbeat Protocol

!!! danger "Connection Timeout"
    The heartbeat mechanism keeps the connection alive. If no heartbeat is sent within 65 seconds, the printer will disconnect the client.

### Configuration

| Parameter | Value |
|-----------|-------|
| Interval | 10 seconds |
| Timeout | 65 seconds (printer disconnects if no heartbeat) |

### Heartbeat Request

Publish to: `elegoo/<sn>/<client_id>/api_request`

```json
{
  "type": "PING"
}
```

### Heartbeat Response

Received on: `elegoo/<sn>/<client_id>/api_response`

```json
{
  "type": "PONG"
}
```

### Implementation Notes

- **Start heartbeat immediately** after successful registration
- **Don't wait for PONG** before sending next PING (fire-and-forget pattern)
- **Missing PONGs** may indicate connection issues
- If connection drops, re-establish from discovery or MQTT connect

### Python Example

```python
import threading
import time

def heartbeat():
    while connected:
        client.publish(
            f"elegoo/{serial_number}/{client_id}/api_request",
            json.dumps({"type": "PING"})
        )
        time.sleep(10)

threading.Thread(target=heartbeat, daemon=True).start()
```

---

## Command Protocol

Commands are JSON messages sent to control the printer.

### Command Message Format

```json
{
  "id": <sequence_number>,
  "method": <method_code>,
  "params": { ... }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | int | Request sequence number (for matching responses) |
| `method` | int | Command method code |
| `params` | object | Command-specific parameters |

### Response Message Format

```json
{
  "id": <sequence_number>,
  "method": <method_code>,
  "result": {
    "error_code": 0,
    ...
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | int | Matches request `id` |
| `method` | int | Matches request `method` |
| `result` | object | Response data including `error_code` |

### Topics

| Action | Topic |
|--------|-------|
| Send command | `elegoo/<sn>/<client_id>/api_request` |
| Receive response | `elegoo/<sn>/<client_id>/api_response` |

### Request ID Management

- Use incrementing integers starting from 1
- Track pending requests by ID
- Implement timeout for responses (5-10 seconds)
- The same ID should not be reused for different requests

---

## Method Reference

### Query Methods (Read)

| Code | Name | Description | Parameters |
|------|------|-------------|------------|
| 1001 | GET_ATTRIBUTES | Get printer info | None |
| 1002 | GET_STATUS | Get full status | None |
| 1036 | PRINT_TASK_LIST | Get print history | `{"page": 1, "page_size": 10}` |
| 1037 | PRINT_TASK_DETAIL | Get task details | `{"uuid": "..."}` |
| 1044 | GET_FILE_LIST | List files | `{"storage_media": "local", "path": "/"}` |
| 1046 | GET_FILE_DETAIL | Get file info | `{"storage_media": "local", "filename": "..."}` |
| 1048 | GET_DISK_INFO | Get storage info | `{"storage_media": "local"}` |
| 2005 | GET_CANVAS_STATUS | Get AMS status | None |

### Print Control Methods

| Code | Name | Description | Parameters |
|------|------|-------------|------------|
| 1020 | START_PRINT | Start print job | See [Start Print](#start-print) |
| 1021 | PAUSE_PRINT | Pause print | None |
| 1022 | STOP_PRINT | Stop/cancel print | None |
| 1023 | RESUME_PRINT | Resume print | None |

### Motion Methods

| Code | Name | Description | Parameters |
|------|------|-------------|------------|
| 1026 | HOME_AXES | Home axes | `{"homed_axes": "xyz"}` |
| 1027 | MOVE_AXES | Move axes | `{"axes": "z", "distance": 10.0}` |

### Temperature Methods

| Code | Name | Description | Parameters |
|------|------|-------------|------------|
| 1028 | SET_TEMPERATURE | Set temps | `{"extruder": 220, "heater_bed": 60}` |

### Peripheral Methods

| Code | Name | Description | Parameters |
|------|------|-------------|------------|
| 1029 | SET_LIGHT | Set LED | `{"power": 1}` (see [Implementation Findings](#light-control-method-1029)) |
| 1030 | SET_FAN_SPEED | Set fans | `{"fan": 255, "aux_fan": 128}` |
| 1031 | SET_PRINT_SPEED | Set speed mode | `{"mode": 1}` |
| 1042 | VIDEO_STREAM | Toggle camera | `{"enable": true}` |

### File Methods

| Code | Name | Description | Parameters |
|------|------|-------------|------------|
| 1047 | DELETE_FILE | Delete file | `{"storage_media": "local", "filename": "..."}` |
| 1057 | DOWNLOAD_FILE | Start download | See [File Upload](#file-upload-via-mqtt) |
| 1058 | CANCEL_DOWNLOAD | Cancel download | `{"filename": "..."}` |

### System Methods

| Code | Name | Description | Parameters |
|------|------|-------------|------------|
| 1038 | DELETE_PRINT_TASK | Delete history | `{"uuid": "..."}` |
| 1043 | UPDATE_PRINTER_NAME | Rename printer | `{"name": "New Name"}` |
| 2004 | SET_AUTO_REFILL | Toggle auto-refill | `{"enable": true}` |

### Event Methods (Printer → Client)

| Code | Name | Description |
|------|------|-------------|
| 6000 | ON_PRINTER_STATUS | Delta status update |
| 6008 | ON_PRINTER_ATTRIBUTES | Attributes changed |

---

## Status Codes Reference

### Machine Status Codes

The primary status indicates what the printer is currently doing.

| Code | Name | Description |
|------|------|-------------|
| 0 | INITIALIZING | Printer booting up |
| 1 | IDLE | Ready, waiting for commands |
| 2 | PRINTING | Print job in progress |
| 3 | FILAMENT_OPERATING | Loading/unloading filament (type 1) |
| 4 | FILAMENT_OPERATING_2 | Loading/unloading filament (type 2) |
| 5 | AUTO_LEVELING | Automatic bed leveling |
| 6 | PID_CALIBRATING | PID auto-tune running |
| 7 | RESONANCE_TESTING | Input shaper calibration |
| 8 | SELF_CHECKING | Hardware self-test |
| 9 | UPDATING | Firmware update in progress |
| 10 | HOMING | Axes homing |
| 11 | FILE_TRANSFERRING | File upload/download active |
| 12 | VIDEO_COMPOSING | Creating timelapse video |
| 13 | EXTRUDER_OPERATING | Extruder maintenance operation |
| 14 | EMERGENCY_STOP | E-stop triggered |
| 15 | POWER_LOSS_RECOVERY | Recovering after power loss |

### Sub-Status Codes

Sub-status provides additional detail within the main status.

#### Printing Sub-Status (status=2)

| Code | Name | Description |
|------|------|-------------|
| 0 | NONE | No specific sub-status |
| 1041 | IDLE_IN_PRINT | Idle within print context |
| 1045 | EXTRUDER_PREHEATING | Nozzle heating up |
| 1096 | EXTRUDER_PREHEATING_2 | Nozzle heating (variant) |
| 1405 | BED_PREHEATING | Bed heating up |
| 1906 | BED_PREHEATING_2 | Bed heating (variant) |
| 2075 | PRINTING | Actively laying down material |
| 2077 | PRINTING_COMPLETED | Print finished successfully |
| 2401 | RESUMING | Resuming from pause |
| 2402 | RESUMING_COMPLETED | Resume complete |
| 2501 | PAUSING | Pause in progress |
| 2502 | PAUSED | Print paused |
| 2505 | PAUSED_2 | Paused (variant) |
| 2503 | STOPPING | Stop in progress |
| 2504 | STOPPED | Print stopped/cancelled |
| 2801 | HOMING | Homing during print |
| 2802 | HOMING_COMPLETED | Mid-print homing done |
| 2901 | AUTO_LEVELING | Leveling during print |
| 2902 | AUTO_LEVELING_COMPLETED | Mid-print leveling done |

#### Filament Operating Sub-Status (status=3,4)

| Code | Name | Description |
|------|------|-------------|
| 1133 | FILAMENT_LOADING | Loading filament |
| 1134 | FILAMENT_LOADING_2 | Loading (phase 2) |
| 1135 | FILAMENT_LOADING_3 | Loading (phase 3) |
| 1136 | FILAMENT_LOADING_COMPLETED | Loading complete |
| 1143 | FILAMENT_PRE_UNLOAD | Preparing to unload |
| 1144 | FILAMENT_UNLOADING | Unloading filament |
| 1145 | FILAMENT_UNLOADING_COMPLETED | Unloading complete |

#### Auto Leveling Sub-Status (status=5)

| Code | Name | Description |
|------|------|-------------|
| 2901 | AL_AUTO_LEVELING | Probing in progress |
| 2902 | AL_AUTO_LEVELING_COMPLETED | Leveling complete |

#### PID Calibrating Sub-Status (status=6)

| Code | Name | Description |
|------|------|-------------|
| 1503 | PID_CALIBRATING | Calibration running |
| 1504 | PID_CALIBRATING_2 | Calibration (phase 2) |
| 1505 | PID_CALIBRATING_COMPLETED | Calibration successful |
| 1506 | PID_CALIBRATING_FAILED | Calibration failed |

#### Resonance Testing Sub-Status (status=7)

| Code | Name | Description |
|------|------|-------------|
| 5934 | RESONANCE_TEST | Test running |
| 5935 | RESONANCE_TEST_COMPLETED | Test successful |
| 5936 | RESONANCE_TEST_FAILED | Test failed |

#### Updating Sub-Status (status=9)

| Code | Name | Description |
|------|------|-------------|
| 2061 | UPDATING_INIT | Update initializing |
| 2071 | UPDATING_1 | Update phase 1 |
| 2072 | UPDATING_2 | Update phase 2 |
| 2073 | UPDATING_3 | Update phase 3 |
| 2074 | UPDATING_COMPLETED | Update successful |
| 2075 | UPDATING_FAILED | Update failed |

#### Homing Sub-Status (status=10)

| Code | Name | Description |
|------|------|-------------|
| 2801 | H_HOMING | Homing in progress |
| 2802 | H_HOMING_COMPLETED | Homing successful |
| 2803 | H_HOMING_FAILED | Homing failed |

#### File Transferring Sub-Status (status=11)

| Code | Name | Description |
|------|------|-------------|
| 3000 | UPLOADING_FILE | Upload in progress |
| 3001 | UPLOADING_FILE_COMPLETED | Upload complete |

#### Extruder Operating Sub-Status (status=13)

| Code | Name | Description |
|------|------|-------------|
| 1061 | EXTRUDER_LOADING | Extruder filament load |
| 1062 | EXTRUDER_UNLOADING | Extruder filament unload |
| 1063 | EXTRUDER_LOADING_COMPLETED | Load complete |
| 1064 | EXTRUDER_UNLOADING_COMPLETED | Unload complete |

### Speed Modes

| Code | Name | Speed Multiplier |
|------|------|------------------|
| 0 | Silent | 50% |
| 1 | Balanced | 100% (default) |
| 2 | Sport | 150% |
| 3 | Ludicrous | 200% |

---

## Data Structures

### Full Status Response

This is the complete status structure returned by method 1002 or event 6000 (first full update).

```json
{
  "id": 1,
  "method": 6000,
  "result": {
    "error_code": 0,

    "machine_status": {
      "status": 2,
      "sub_status": 2075,
      "exception_status": [],
      "progress": 45
    },

    "print_status": {
      "filename": "benchy.gcode",
      "uuid": "b52af24c-764e-4092-8a50-00e5f8f02b46",
      "current_layer": 225,
      "total_layer": 500,
      "print_duration": 3600,
      "total_duration": 8000,
      "remaining_time_sec": 4400,
      "progress": 45
    },

    "extruder": {
      "temperature": 215.0,
      "target": 220,
      "filament_detect_enable": 1,
      "filament_detected": 1
    },

    "heater_bed": {
      "temperature": 58.5,
      "target": 60
    },

    "ztemperature_sensor": {
      "temperature": 33.0,
      "measured_max_temperature": 0,
      "measured_min_temperature": 0
    },

    "fans": {
      "fan": {
        "speed": 255,
        "rpm": 5000
      },
      "aux_fan": {
        "speed": 178,
        "rpm": 3500
      },
      "box_fan": {
        "speed": 25,
        "rpm": 800
      },
      "heater_fan": {
        "speed": 255,
        "rpm": 4500
      },
      "controller_fan": {
        "speed": 255,
        "rpm": 4000
      }
    },

    "led": {
      "status": 1
    },

    "gcode_move_inf": {
      "x": 88.148,
      "y": 139.946,
      "z": 1.6,
      "e": 138.87,
      "speed": 9019,
      "speed_mode": 1
    },

    "toolhead": {
      "homed_axes": "xyz"
    },

    "external_device": {
      "camera": true,
      "u_disk": false,
      "type": "0303"
    }
  }
}
```

### Field Descriptions

#### machine_status

| Field | Type | Description |
|-------|------|-------------|
| `status` | int | Machine status code (see [Machine Status Codes](#machine-status-codes)) |
| `sub_status` | int | Sub-status code (see [Sub-Status Codes](#sub-status-codes)) |
| `exception_status` | array | List of active error codes |
| `progress` | int | Print progress 0-100 (also in print_status) |

#### print_status

| Field | Type | Description |
|-------|------|-------------|
| `filename` | string | Current print file name |
| `uuid` | string | Unique print task identifier |
| `current_layer` | int | Current layer being printed |
| `total_layer` | int | Total layers in print |
| `print_duration` | int | Elapsed time in seconds |
| `total_duration` | int | Estimated total time in seconds |
| `remaining_time_sec` | int | Estimated remaining time in seconds |
| `progress` | int | Progress percentage 0-100 |

#### extruder

| Field | Type | Description |
|-------|------|-------------|
| `temperature` | float | Current nozzle temperature (C) |
| `target` | float | Target nozzle temperature (C) |
| `filament_detect_enable` | int | 1=sensor enabled, 0=disabled |
| `filament_detected` | int | 1=filament present, 0=no filament |

#### heater_bed

| Field | Type | Description |
|-------|------|-------------|
| `temperature` | float | Current bed temperature (C) |
| `target` | float | Target bed temperature (C) |

#### ztemperature_sensor (Chamber/Box)

| Field | Type | Description |
|-------|------|-------------|
| `temperature` | float | Current chamber temperature (C) |
| `measured_max_temperature` | float | Max recorded temp |
| `measured_min_temperature` | float | Min recorded temp |

#### fans

Each fan object contains:

| Field | Type | Description |
|-------|------|-------------|
| `speed` | int | PWM value 0-255 (NOT percentage) |
| `rpm` | int | Actual measured RPM |

Fan types:

- `fan` - Part cooling fan (model fan)
- `aux_fan` - Auxiliary/chamber circulation fan
- `box_fan` - Enclosure exhaust fan
- `heater_fan` - Hotend heatsink fan
- `controller_fan` - Electronics cooling fan

**Converting speed to percentage:**
```python
percentage = round(speed / 255 * 100)
```

#### led

| Field | Type | Description |
|-------|------|-------------|
| `status` | int | 0=off, 1=on (may also be brightness 0-255) |

#### gcode_move_inf

| Field | Type | Description |
|-------|------|-------------|
| `x` | float | X axis position (mm) |
| `y` | float | Y axis position (mm) |
| `z` | float | Z axis position (mm) |
| `e` | float | Extruder position (mm of filament) |
| `speed` | int | Current move speed (mm/min) |
| `speed_mode` | int | Speed mode 0-3 (see [Speed Modes](#speed-modes)) |

#### toolhead

| Field | Type | Description |
|-------|------|-------------|
| `homed_axes` | string | Which axes are homed ("", "x", "xy", "xyz") |

#### external_device

| Field | Type | Description |
|-------|------|-------------|
| `camera` | bool | Camera connected |
| `u_disk` | bool | USB drive connected |
| `type` | string | Device type identifier |

### Attributes Response

```json
{
  "id": 1,
  "method": 1001,
  "result": {
    "error_code": 0,
    "hostname": "My Printer",
    "machine_model": "Centauri Carbon 2",
    "sn": "CC2ABCD1234567890",
    "ip": "192.168.1.100",
    "mac": "AA:BB:CC:DD:EE:FF",
    "protocol_version": "1.0.0",
    "hardware_version": "1.0",
    "software_version": {
      "ota_version": "1.0.5.2",
      "mcu_version": "00.00.00.00",
      "soc_version": ""
    },
    "resolution": "1920x1080",
    "xyz_size": "220x220x250",
    "network_type": "wifi",
    "usb_connected": false,
    "camera_connected": true,
    "remaining_memory": 1073741824,
    "max_video_connections": 1,
    "video_connections": 0
  }
}
```

### Field Name Variations

!!! warning "Firmware Variations"
    Different firmware versions may use different field names. Implementations should handle both:

| Official Name | Alternative | Notes |
|---------------|-------------|-------|
| `gcode_move_inf` | `gcode_move` | Position/speed data |
| `gcode_move_inf.e` | `gcode_move.extruder` | Extruder position |
| `toolhead` | `tool_head` | Toolhead info |
| `ztemperature_sensor` | `chamber` | Chamber temperature |

---

## Delta Status Updates

CC2 uses delta updates to minimize bandwidth. Only changed fields are sent.

### How Delta Updates Work

1. **Initial Full Status**: On connection or explicit request (method 1002), printer sends complete status
2. **Incremental Updates**: Subsequent event 6000 messages contain only changed fields
3. **Client Merging**: Client must merge delta into cached full status
4. **Continuity Tracking**: Track message IDs to detect missed updates

### Delta Update Example

Full status has been received. Then printer sends:

```json
{
  "id": 42,
  "method": 6000,
  "result": {
    "error_code": 0,
    "machine_status": {
      "progress": 46
    },
    "print_status": {
      "current_layer": 230,
      "print_duration": 3650
    },
    "extruder": {
      "temperature": 219.5
    }
  }
}
```

Only `progress`, `current_layer`, `print_duration`, and `extruder.temperature` changed.

### Deep Merge Algorithm

```python
def deep_merge(base: dict, update: dict) -> dict:
    """Recursively merge update into base."""
    result = base.copy()
    for key, value in update.items():
        if key in result and isinstance(result[key], dict) and isinstance(value, dict):
            result[key] = deep_merge(result[key], value)
        else:
            result[key] = value
    return result

# Usage
cached_status = deep_merge(cached_status, delta_update["result"])
```

### Continuity Checking

Track the `id` field to detect missed updates:

```python
class StatusManager:
    def __init__(self):
        self.last_id = None
        self.non_continuous_count = 0
        self.MAX_NON_CONTINUOUS = 5

    def process_update(self, message):
        current_id = message.get("id")

        if self.last_id is not None:
            if current_id != self.last_id + 1:
                self.non_continuous_count += 1
                if self.non_continuous_count >= self.MAX_NON_CONTINUOUS:
                    self.request_full_status()
                    self.non_continuous_count = 0
            else:
                self.non_continuous_count = 0

        self.last_id = current_id
```

### When to Request Full Status

- After 5+ non-continuous ID gaps
- After reconnection
- If critical fields appear missing
- Periodically (every 5-10 minutes) as safety measure

---

## HTTP API

The CC2 provides an HTTP API for operations not suited to MQTT.

### Base URL

```
http://<printer_ip>:8080
```

### Authentication

All HTTP requests require authentication via the `X-Token` header or query parameter:

```
X-Token: <access_code>
```

Or:

```
?X-Token=<access_code>
```

If no access code is set, use `123456`.

### Get System Info

```
GET /system/info?X-Token=<token>
```

Response:
```json
{
  "error_code": 0,
  "system_info": {
    "sn": "CC2ABCD1234567890"
  }
}
```

### List Files

```
GET /files?storage_media=local&path=/&X-Token=<token>
```

Response:
```json
{
  "error_code": 0,
  "files": [
    {
      "name": "benchy.gcode",
      "size": 1234567,
      "modified": 1706900000,
      "type": "file"
    },
    {
      "name": "models",
      "type": "directory"
    }
  ]
}
```

### Download File

```
GET /download?file_name=<path>&X-Token=<token>
GET /download/sdcard?file_name=<path>&X-Token=<token>
GET /download/udisk?file_name=<path>&X-Token=<token>
```

Returns raw file contents.

### Upload File

Chunked upload for large files:

```
PUT /upload
Content-Type: application/octet-stream
Content-Range: bytes 0-1048575/5242880
X-File-Name: model.gcode
X-File-MD5: abc123def456...
X-Token: <token>

<binary chunk data>
```

| Header | Description |
|--------|-------------|
| `Content-Range` | Byte range being uploaded |
| `X-File-Name` | Target filename |
| `X-File-MD5` | MD5 hash of complete file |
| `X-Token` | Authentication |

!!! info "Upload Constraints"
    - Maximum chunk size: 1 MB (1048576 bytes)
    - Calculate MD5 before starting upload
    - Use persistent HTTP connections for efficiency
    - Last chunk completes the upload

### Upload Response

```json
{
  "error_code": 0,
  "received": 1048576,
  "total": 5242880
}
```

---

## File Operations

### Storage Media Types

| Value | Description |
|-------|-------------|
| `local` | Internal storage |
| `u-disk` | USB drive |
| `sd-card` | SD card |

### File List via MQTT

```json
{
  "id": 1,
  "method": 1044,
  "params": {
    "storage_media": "local",
    "path": "/",
    "page": 1,
    "page_size": 50
  }
}
```

Response:
```json
{
  "id": 1,
  "method": 1044,
  "result": {
    "error_code": 0,
    "total": 25,
    "files": [
      {
        "name": "benchy.gcode",
        "size": 1234567,
        "modified": 1706900000
      }
    ]
  }
}
```

### File Upload via MQTT

MQTT-based upload initiates transfer, actual data sent via HTTP:

```json
{
  "id": 1,
  "method": 1057,
  "params": {
    "storage_media": "local",
    "filename": "model.gcode",
    "size": 5242880,
    "md5": "abc123def456..."
  }
}
```

### Delete File

```json
{
  "id": 1,
  "method": 1047,
  "params": {
    "storage_media": "local",
    "filename": "old_model.gcode"
  }
}
```

### Disk Info

```json
{
  "id": 1,
  "method": 1048,
  "params": {
    "storage_media": "local"
  }
}
```

Response:
```json
{
  "id": 1,
  "method": 1048,
  "result": {
    "error_code": 0,
    "total_bytes": 8589934592,
    "free_bytes": 4294967296,
    "used_bytes": 4294967296
  }
}
```

---

## Video Streaming

The CC2 camera provides MJPEG streaming.

### Enable Video Stream

```json
{
  "id": 1,
  "method": 1042,
  "params": {
    "enable": true
  }
}
```

### Stream URL

```
http://<printer_ip>:8080/?action=stream
```

### Stream Characteristics

- Format: MJPEG (Motion JPEG)
- Resolution: Varies by camera
- Max connections: Usually 1 (check `max_video_connections` in attributes)
- No authentication required for stream itself

### Example: Display Stream

```python
import cv2

stream_url = f"http://{printer_ip}:8080/?action=stream"
cap = cv2.VideoCapture(stream_url)

while True:
    ret, frame = cap.read()
    if ret:
        cv2.imshow('Printer Camera', frame)
    if cv2.waitKey(1) & 0xFF == ord('q'):
        break

cap.release()
```

---

## Canvas/AMS System

The Canvas is Elegoo's Automatic Material System (similar to Bambu Lab AMS).

### Get Canvas Status

```json
{
  "id": 1,
  "method": 2005,
  "params": {}
}
```

### Canvas Status Response

```json
{
  "id": 1,
  "method": 2005,
  "result": {
    "error_code": 0,
    "canvas_info": {
      "active_canvas_id": 0,
      "active_tray_id": 0,
      "auto_refill": false,
      "canvas_list": [
        {
          "canvas_id": 1,
          "connected": 1,
          "tray_list": [
            {
              "tray_id": 1,
              "brand": "ELEGOO",
              "filament_type": "PLA",
              "filament_name": "Generic PLA",
              "filament_color": "FFFFFF",
              "nozzle_temp_min": 190,
              "nozzle_temp_max": 220,
              "bed_temp_min": 50,
              "bed_temp_max": 60,
              "status": 1
            },
            {
              "tray_id": 2,
              "brand": "ELEGOO",
              "filament_type": "PETG",
              "filament_name": "Generic PETG",
              "filament_color": "FF0000",
              "nozzle_temp_min": 230,
              "nozzle_temp_max": 250,
              "bed_temp_min": 70,
              "bed_temp_max": 85,
              "status": 1
            }
          ]
        }
      ]
    }
  }
}
```

### Canvas Fields

| Field | Description |
|-------|-------------|
| `active_canvas_id` | Currently selected Canvas unit (0=none) |
| `active_tray_id` | Currently selected tray (0=none) |
| `auto_refill` | Auto-switch when filament runs out |
| `canvas_id` | Canvas unit number (1-4 typically) |
| `connected` | 1=connected, 0=disconnected |
| `tray_id` | Tray slot number (1-4 typically) |
| `filament_color` | Hex color code (RRGGBB) |
| `status` | 1=filament present, 0=empty |

### Set Auto Refill

```json
{
  "id": 1,
  "method": 2004,
  "params": {
    "enable": true
  }
}
```

### Printing with Canvas

When starting a print with Canvas, include slot mapping:

```json
{
  "id": 1,
  "method": 1020,
  "params": {
    "storage_media": "local",
    "filename": "multicolor.gcode",
    "config": {
      "slot_map": [
        {"slot": 1, "canvas_id": 1, "tray_id": 1},
        {"slot": 2, "canvas_id": 1, "tray_id": 2}
      ]
    }
  }
}
```

---

## Print Job Lifecycle

### Print State Machine

```
                          ┌─────────────┐
                          │    IDLE     │
                          └──────┬──────┘
                                 │ START_PRINT
                                 ▼
                ┌────────────────────────────────┐
                │           PRINTING             │
                │  ┌──────────────────────────┐  │
                │  │  PREHEATING              │  │
                │  │    ↓                     │  │
                │  │  HOMING (optional)       │  │
                │  │    ↓                     │  │
                │  │  LEVELING (optional)     │  │
                │  │    ↓                     │  │
                │  │  PRINTING ◄──────────┐   │  │
                │  │    │                 │   │  │
                │  │    ├── PAUSING ──► PAUSED  │  │
                │  │    │                 │   │  │
                │  │    │        RESUMING─┘   │  │
                │  └────┼─────────────────────┘  │
                │       │                        │
                └───────┼────────────────────────┘
                        │
          ┌─────────────┼─────────────┐
          │             │             │
          ▼             ▼             ▼
    ┌──────────┐  ┌──────────┐  ┌──────────┐
    │ COMPLETE │  │ STOPPING │  │  ERROR   │
    └──────────┘  └────┬─────┘  └──────────┘
                       │
                       ▼
                 ┌──────────┐
                 │ STOPPED  │
                 └──────────┘
```

### Start Print

```json
{
  "id": 1,
  "method": 1020,
  "params": {
    "storage_media": "local",
    "filename": "benchy.gcode",
    "config": {
      "delay_video": false,
      "printer_check": true,
      "print_layout": "A",
      "bedlevel_force": false,
      "slot_map": []
    }
  }
}
```

#### Config Options

| Field | Type | Description |
|-------|------|-------------|
| `delay_video` | bool | Delay start until video streaming active |
| `printer_check` | bool | Run pre-print checks |
| `print_layout` | string | Layout option (model-specific) |
| `bedlevel_force` | bool | Force bed leveling before print |
| `slot_map` | array | Canvas/AMS filament mapping |

### Pause Print

```json
{
  "id": 1,
  "method": 1021,
  "params": {}
}
```

### Resume Print

```json
{
  "id": 1,
  "method": 1023,
  "params": {}
}
```

### Stop Print

```json
{
  "id": 1,
  "method": 1022,
  "params": {}
}
```

### Print Progress Monitoring

Monitor these fields during printing:

| Field | Location | Description |
|-------|----------|-------------|
| `status` | `machine_status.status` | Should be 2 (PRINTING) |
| `sub_status` | `machine_status.sub_status` | Detailed state |
| `progress` | `machine_status.progress` or `print_status.progress` | 0-100% |
| `current_layer` | `print_status.current_layer` | Current layer |
| `remaining_time_sec` | `print_status.remaining_time_sec` | ETA in seconds |

---

## Error Handling

### Error Code Reference

| Code | Name | Description | Recovery |
|------|------|-------------|----------|
| 0 | SUCCESS | Operation completed | N/A |
| 109 | FILAMENT_RUNOUT | No filament detected | Load filament, resume |
| 1000 | TOKEN_FAILED | Authentication failed | Check access code |
| 1001 | UNKNOWN_INTERFACE | Unknown command | Check method code |
| 1002 | FOLDER_OPEN_FAILED | Cannot access folder | Check path |
| 1003 | INVALID_PARAMETER | Bad parameter | Check params |
| 1004 | FILE_WRITE_FAILED | Cannot write file | Check disk space |
| 1005 | TOKEN_UPDATE_FAILED | Token refresh failed | Re-authenticate |
| 1006 | MOS_UPDATE_FAILED | MOS update failed | Retry |
| 1007 | FILE_DELETE_FAILED | Cannot delete file | Check file exists |
| 1008 | RESPONSE_EMPTY | No data returned | Retry |
| 1009 | PRINTER_BUSY | Printer occupied | Wait, retry |
| 1010 | NOT_PRINTING | No active print | Check state first |
| 1011 | FILE_COPY_FAILED | Copy failed | Check disk space |
| 1012 | TASK_NOT_FOUND | Print task missing | Refresh task list |
| 1013 | DATABASE_FAILED | DB error | Internal error |
| 1021 | PRINT_FILE_NOT_FOUND | File doesn't exist | Upload file |
| 1026 | MISSING_BED_LEVELING | No mesh data | Run leveling |
| 9000 | FILE_OFFSET_MISMATCH | Upload offset wrong | Restart upload |
| 9001 | FILE_OPEN_FAILED | Cannot open file | Check filename |
| 9002 | FILE_WRITE_ERROR | Write error | Check disk |
| 9003 | FILE_SEEK_FAILED | Seek error | Retry |
| 9004 | MD5_FAILED | Checksum mismatch | Re-upload |
| 9005 | CANCEL_NOT_NEEDED | Nothing to cancel | Ignore |
| 9006 | CANCEL_FAILED | Cancel failed | Force retry |
| 9007 | PATH_NOT_EXISTS | Path not found | Check path |
| 9008 | MD5_SYSTEM_ERROR | System MD5 error | Retry |
| 9009 | MD5_READ_ERROR | File read error | Retry |
| 9999 | UNKNOWN_ERROR | Unclassified error | Check logs |

### Exception Status

The `machine_status.exception_status` array contains active errors:

```json
{
  "machine_status": {
    "exception_status": [109, 1026]
  }
}
```

This indicates both filament runout (109) and missing bed leveling (1026).

### Error Recovery Patterns

#### Filament Runout
1. Detect error 109 in exception_status
2. Notify user
3. User loads filament
4. Clear exception (may require print resume)
5. Issue RESUME_PRINT command

#### Authentication Error
1. Detect error 1000
2. Re-prompt user for access code
3. Reconnect with new credentials

#### Printer Busy
1. Detect error 1009
2. Wait 5-10 seconds
3. Retry command
4. After 3 failures, check printer status

---

## Security Considerations

### Access Code

- When `token_status=1` in discovery, an access code is required
- The access code replaces the default password `123456`
- Access codes are set through the printer's touchscreen
- Store access codes securely (not in plain text logs)

### Network Security

!!! danger "Unencrypted Traffic"
    CC2 uses **unencrypted** MQTT and HTTP. All traffic is visible on the local network.

- Recommend keeping printers on isolated IoT network
- Do not expose ports 1883 or 8080 to the internet

### Client Limits

- Maximum ~4 concurrent MQTT clients
- Malicious clients could DoS by filling all slots
- Implement clean disconnection on application exit

### Best Practices

1. **Never hardcode access codes** - prompt user or use secure storage
2. **Use network isolation** - separate VLAN for IoT devices
3. **Implement connection timeouts** - don't hold connections indefinitely
4. **Handle disconnection gracefully** - free up client slots

---

## CC1 vs CC2 Comparison

| Feature | CC1 (Centauri Carbon) | CC2 (Centauri Carbon 2) |
|---------|----------------------|-------------------------|
| **Discovery** |
| Port | 3000 | 52700 |
| Message | `M99999` | `{"id":0,"method":7000}` |
| Protocol | TCP | UDP |
| **Communication** |
| Transport | WebSocket | MQTT |
| Broker Location | External (HA) | **On Printer** |
| Client Role | Server | **Client** |
| **Authentication** |
| Registration | Not required | **Required** |
| Password | None | `123456` or access code |
| **Connection** |
| Heartbeat | Not required | **Every 10s** |
| Max Clients | Unlimited | **~4** |
| Timeout | None | **65 seconds** |
| **Status** |
| Update Type | Full | **Delta** |
| Message Format | Proprietary | JSON |
| **Features** |
| Canvas/AMS | No | Yes |
| Video Stream | Varies | MJPEG on :8080 |

---

## Implementation Checklist

Use this checklist when implementing CC2 support:

### Discovery

- [ ] UDP broadcast to port 52700
- [ ] Handle multiple printer responses
- [ ] Parse discovery response fields
- [ ] Store serial number for topics
- [ ] Handle token_status for auth

### Connection

- [ ] MQTT 3.1.1 client
- [ ] Generate unique client_id
- [ ] Handle authentication (default + access code)
- [ ] Connection error handling
- [ ] Automatic reconnection

### Registration

- [ ] Subscribe to register_response topic
- [ ] Publish registration request
- [ ] Handle success/failure responses
- [ ] Timeout handling (3 seconds)
- [ ] Retry logic for "too many clients"

### Heartbeat

- [ ] 10-second interval timer
- [ ] PING message publishing
- [ ] PONG response handling
- [ ] 65-second timeout detection
- [ ] Reconnection on timeout

### Status

- [ ] Subscribe to api_status topic
- [ ] Parse full status structure
- [ ] Implement delta merge
- [ ] Track message IDs for continuity
- [ ] Request full status on gaps

### Commands

- [ ] Command request formatting
- [ ] Response matching by ID
- [ ] Error code handling
- [ ] Timeout handling

### File Operations

- [ ] File listing (MQTT or HTTP)
- [ ] File upload (chunked HTTP)
- [ ] File download
- [ ] MD5 verification

### Peripheral Control

- [ ] Temperature control
- [ ] Fan speed control (0-255)
- [ ] Light control
- [ ] Speed mode control

### Print Control

- [ ] Start print
- [ ] Pause/Resume
- [ ] Stop
- [ ] Progress monitoring

### Advanced

- [ ] Canvas/AMS status
- [ ] Video streaming
- [ ] Print history
- [ ] Firmware version handling

---

## Troubleshooting

### Connection Issues

**Problem**: Discovery times out

- Check printer is powered on and connected to network
- Verify you're on the same network/subnet
- Check firewall allows UDP port 52700
- Try direct IP instead of broadcast

**Problem**: MQTT connection refused

- Verify printer IP is correct
- Check port 1883 is not blocked
- Verify credentials (check token_status)
- Check another client isn't blocking

**Problem**: Registration fails with "too many clients"

- Disconnect other clients (slicer, app)
- Wait 65 seconds for timeout to expire
- Restart printer to clear all connections

**Problem**: Heartbeat timeout

- Check network stability
- Verify PING messages being sent
- Check for PONG responses
- Reduce heartbeat interval if needed

### Data Issues

**Problem**: Missing status fields

- Request full status (method 1002)
- Check firmware version
- Handle field variations defensively

**Problem**: Delta updates not applying

- Verify deep merge implementation
- Check for ID continuity
- Request full status periodically

**Problem**: Wrong temperature/progress values

- Verify you're merging deltas correctly
- Check field paths in nested structure

### Print Issues

**Problem**: Start print fails with "file not found"

- Verify filename is correct
- Check storage_media value
- Upload file first

**Problem**: Print doesn't resume after pause

- Check printer isn't in error state
- Verify sub_status is PAUSED
- Clear any exception_status errors first

---

## References

### Official Sources

- [elegoo-link](https://github.com/ELEGOO-3D/elegoo-link) - Official Elegoo network library (primary source)
- [ElegooSlicer](https://github.com/ELEGOO-3D/ElegooSlicer) - Official slicer application
- [elegoo-fdm-web](https://github.com/ELEGOO-3D/elegoo-fdm-web) - Web interface releases

### Community Projects

- [elegoo-homeassistant](https://github.com/danielcherubini/elegoo-homeassistant) - Home Assistant integration
- [OpenCentauri](https://opencentauri.org) - Community documentation project

### Related Documentation

- [MQTT 3.1.1 Specification](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/mqtt-v3.1.1.html)
- [Paho MQTT Python](https://eclipse.dev/paho/files/paho.mqtt.python/html/index.html)

---

## Implementation Findings

This section documents discrepancies and additional findings discovered during real-world implementation that differ from or extend the main documentation.

### Light Control (Method 1029)

**Documentation stated**: `{"brightness": 255}` parameter

**Actual behavior**: The web interface uses `power` parameter:

```json
{
  "id": 1,
  "method": 1029,
  "params": {
    "power": 1
  }
}
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `power` | int | 0 = off, 1 = on |

The `brightness` parameter may work on some firmware versions, but `power` is what the official web interface uses.

### Total Layers Not in Delta Updates

!!! warning "Missing Field"
    The `total_layer` field in `print_status` is often missing from delta status updates (method 6000).

**Explanation**: Since `total_layer` doesn't change during printing, the CC2 protocol omits it from delta updates to save bandwidth. The field is only reliably present in:

1. Full status response (method 1002) at print start
2. File details response (method 1046)

**Workaround**: When `total_layer` is missing from `print_status`, fetch file details:

```json
{
  "id": 1,
  "method": 1046,
  "params": {
    "storage_media": "local",
    "filename": "benchy.gcode"
  }
}
```

Response includes:

```json
{
  "id": 1,
  "method": 1046,
  "result": {
    "TotalLayers": 500,
    "layer": 500,
    ...
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `TotalLayers` | int | Total layers in the gcode file |
| `layer` | int | Same as TotalLayers (alternate field name) |

### Field Name Variations in File Details

The file details response (method 1046) may use different field names across firmware versions:

| Standard Name | Variations |
|--------------|------------|
| `TotalLayers` | `layer`, `total_layer` |

Always check for multiple field names when parsing file details.

### Begin Time / End Time

!!! info "Calculated Fields"
    `begin_time` and `end_time` are not provided in live print status.

**Available fields in print_status**:

- `print_duration` - Elapsed time in seconds
- `remaining_time_sec` - Estimated remaining time in seconds
- `total_duration` - Estimated total print time in seconds

**Calculating begin_time**: `current_time - print_duration`

**Calculating end_time**: `current_time + remaining_time_sec`

!!! note
    `begin_time` and `end_time` ARE provided in historical task data (method 1036/1037) but not in live status.
