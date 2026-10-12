# Pins & Communication between ESP32 and Radio

The following table describes the important pins on the easyRadio.
Input/Output directions are from the perspective of the radio.

| Pin # | Name | Type | Note |
| ----- | ---- | ---- | ---- |
| 2 | Busy | Digital Out | High - Transceiver is busy |
| 3 | SDO | (UART) Rx Serial Data Out |
| 4 | SDI | (UART) Tx Serial Data In |
| 5 | Host Ready | Digital In | Low - Host ready to receive data |
| 6 | Vcc | Supply Voltage | +2.4V to +6V, must be clean |
| 7 | Gnd | Ground Voltage ||
| 23 | RF Gnd | RF Ground | Connect to antenna and local ground plane |
| 24 | RF | 50R RF Input/Output | Connect to suitable antenna via 50R PCB trace or UFL Connector|

Here is a tentative list of pin connections from the host (ESP32) to the Radio:
| Radio Pin | ESP32 Pin | Note |
| --------- | --------- | ---- |
| 2 | Any GPIO Pin ||
| 3 | Any UART Rx Pin ||
| 4 | Any UART Tx Pin ||
| 5 | Any GPIO Pin ||
| 6 | Could be 3.3V or 5V | Are the pins on the board out pins? |
| 7 | Any Gnd | Same Gnd as 6 |
| 23 | Local Gnd ||

Additionally the ESP32 itself will have pins going out to:
 - The CANBUS (called TWAI in ESP32 docs) wires should be fine
   going to any GPIO pin. Need two pins for this.
 - Power and ground will come from the car's 24V network.

## Pin Usage Notes
Some GPIO pins have special consideration for their usage according to pages
24-27 of the esp32-c6 datasheet. The pins are separated into four categories, we care
about P3 and P4 pins. Priority 2 (P3) pins are free for any GPIO use case. P3
pins are pins that have use cases important for either debugging or power and
should be used with caution. P4 pins are reserved for communication with
off-package flash.

P3 pins:
 - 4, 5, 8, 9, 15: strapping pins, configure some behaviors based upon their
   values at startup
    * see pg 33 for more info
 - 12, 13: USB serial, JTAG interface
 - 4, 5, 6, 7: JTAG interface
 - 16, 17: `UART0` interface (UART0 is used for debugging)
 - 27: `VDD_SPI` default power supply pin for off-package flash

