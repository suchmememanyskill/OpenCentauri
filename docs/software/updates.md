# Updates

The Centauri Carbon during startup checks [an endpoint on chituiot.com](https://mms.chituiot.com/mainboardVersionUpdate/getInfo.do7?machineType=ELEGOO%20Centauri%20Carbon&machineId=0&version=1.1.0&lan=en&firmwareType=1) to check if a new firmware update is available.

!!! question "Speculation"
    These firmware updates seem to be encrypted (very high entropy).

    I suspect, looking at the coredump, that the result after decryption is a bz2 archive.

## Firmware update archive

### v1.1.18 (Released 31/03/2025)
```json
{
    "code": "000000",
    "messages": null,
    "data": {
        "update": true,
        "version": "1.1.18",
        "packageUrl": "https://download.chitubox.com/chitusystems/chitusystems/public/printer/firmware/release/1/ca8e1d9a20974a5896f8f744e780a8a7/1/1.1.18/2025-03-31/74406d43dc314af7a174dba70487ac2b.bin",
        "firmwareType": 1,
        "packageHash": "ab50592f9c7bbac725b7c75ff1213fc0",
        "updateStrategy": 1,
        "log": "Fixes：\n1. Fixed flickering issue in the video stream.\n2. Added Korean to the language options.\n3. Fixed several UI logic bugs.\n4. Fixed an issue where time-lapse videos couldn't be exported or generated in some cases.\n5. Fixed abnormal strain gauge detection during printing.\n6. Adjusted the extruder position after stopping a print.\n7. Resolved an issue where the printer could get stuck at the target temperature and fail to start the print.",
        "timeMS": 1746044457465,
        "dataInfoId": "a4d7da0841de41428484f7a17529ebce"
    },
    "success": true
}
```