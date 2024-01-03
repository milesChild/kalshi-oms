# The Client Protocol

## Problem

Ideally, the client and client-server communicate via a low-level information language, defined in bytes. The alternatives, such as JSON strings, are convenient for writing clients quickly, but they introduce significant additional computation and wire traffic (as the sheer number of bytes per message is magnified). This document will specify our attempt at designing a protocol that condenses all necessary communication (e.g., order entry, order confirmation, etc.) into a minimal number of bytes.

## High-level Diagram

+----------+----------+-----------------------------+
| Msg Len  | Msg Type | Message Fields              |
| (1 byte) | (1 byte) | (variable number of bytes)  |
+----------+----------+-----------------------------+