# The Client Protocol

## Problem

Ideally, the client and client-server communicate via a low-level information language, defined in bytes. The alternatives, such as JSON strings, are convenient for writing clients quickly, but they introduce significant additional computation and wire traffic (as the sheer number of bytes per message is magnified). This document will specify our attempt at designing a protocol that condenses all necessary communication (e.g., order entry, order confirmation, etc.) into a minimal number of bytes.

## High-level Diagram
```
+----------+----------+-----------------------------+
| Msg Len  | Msg Type | Custom Fields              |
| (1 byte) | (1 byte) | (variable number of bytes)  |
+----------+----------+-----------------------------+
```
## Fields

### - Msg Len
The first byte of every message denotes the length of the variable-length portion of the message as a standard unsigned 8-bit integer (equivalent to the Rust u8 type).

### - Msg Type
The next byte specifies (via a u8 as above) which type of message is being written according to the below reference:

| Msg Type    | Value (decimal) | Value (8-bit binary) |
| ----------- | --------------- | -------------------- |
| CreateOrder | 0 | 00000000 |
| CancelOrder | 1 | 00000001 |
| ConfirmOrder | 2 | 00000010 |
| ConfirmCancel | 3 | 00000011 |
| Fill | 4 | 00000100 |