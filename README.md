# kalshi-oms
low-latency order management system for kalshi
_____________

# introduction
_____________

the kalshi OMS is a low-latency, rust-based platform that provides a simple, reliable, and fast interface between external trading clients and kalshi's binary exchange. any number of trading clients can establish a tcp connection to the kalshi OMS for standardized order placement, updating, cancellation, and status-update retreival.

read the **quick start** section for instructions on how to start an instance of the kalshi OMS. read the **connecting a trading client** section for instructions on how to connect your own trading client to an active instance of the kalshi OMS and guidance for expected behavior.

the kalshi OMS was developed and is maintained by the developers of the [kalshi-mdp](https://github.com/rothcharlie1/kalshi-mdp), a rust-based market data platform.

# quick start
_____________

before starting the application, we recommend versioning up and rebuilding the project:
```
git checkout master
git pull
cargo build --release
```

**to start the entire OMS: in your terminal, run:**
```
./start-kalshi-oms
```

**to start the client-server: in your terminal, run:**

```
RUST_LOG=<desired_log_level> ./target/release/kalshi-client-server
```

**to start the exchange server: in your terminal, run:**
```
RUST_LOG=<desired_log_level> ./target/release/kalshi-exchange-server
```

the default log-level is info. we recommend using debug for testing and info for production.

# connecting a trading client
_____________

# high-level implementation
_____________

**client-server/**

trading client-facing side of the OMS. Allows multiple clients to establish a tcp connection to the OMS over which they may submit requests to place new orders, update/cancel existing orders, and receive updates on orders they've placed.

**exchange-server/**

exchange-facing side of the OMS. Provides a two-way ordering interface between the client-server and kalshi, joined with a shared in-memory messaging queue. Our implementation uses a series of RabbitMQ message queues for each unique message struct to/from the client-server and exchange server.

this OMS currently supports ordering via kalshi's REST API and fill update querying via kalshi's websocket market data feed. future implementations will offer the option for advanced users to select a FIX exchange-server that uses kalshi's nascent FIX API for all information exchange between the OMS and kalshi.