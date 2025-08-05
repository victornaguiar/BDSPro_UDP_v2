# Running the Experiments

Both channels (TCP and UDP) were tested under various network conditions. During testing, latency, bandwidth, and 
packet loss rate were manipulated.

The tests were conducted on a MacBook Pro with an M1 chip (2021). To simulate network disturbances, **Packet Filter (pf)** 
and **dummynet** were used, as they are compatible with macOS. Equivalent tools exist for other operating systems that 
provide similar functionality.

---

## Simulating Network Disturbances

First, enable the Packet Filter and configure dummynet using `sudo`:

```shell
sudo pfctl -e
(cat /etc/pf.conf && echo "dummynet-anchor \"mop\"" && echo "anchor \"mop\"") | sudo pfctl -f -
```

Next, create a dummynet pipe to which network traffic will be redirected. The following commands apply the redirection rules for both TCP and UDP on port 8001:

```shell
# TCP
echo "dummynet in quick proto tcp from any to any port 8001 pipe 1" | sudo pfctl -a mop -f -

# UDP
echo "dummynet in quick proto udp from any to any port 8001 pipe 1" | sudo pfctl -a mop -f -
```
You can now manipulate the network conditions by configuring the pipe. The following commands can be used to introduce delay, bandwidth limitation, and packet loss:

```shell
# Set delay in milliseconds
sudo dnctl pipe 1 config delay 1000ms

# Set bandwidth in Kbit/s
sudo dnctl pipe 1 config bw 10Kbit/s

# Set packet loss rate (as a decimal)
sudo dnctl pipe 1 config plr 0.05
```

Before each test case, the network conditions were reconfigured accordingly. 
All traffic manipulations were applied to port `8001`.

## Executing Test Cases
### TCP Channel
For the TCP channel, the test was executed using examples/simple_example.yml. This configuration sets up a sender node
at `localhost:8000` and a receiver node at `localhost:8001`. Note that the sender waits 2 seconds before sending queries 
to allow time for the receiver to initialize.

Open two terminal windows and run:

```shell
# Sender
cargo run --bin test-node -- simple_config.yml 0

# Receiver
cargo run --bin test-node -- simple_config.yml 1
```

### UDP Channel
The UDP tests consist of a sender and a receiver using the same port. First start the receiver, then the sender. 
The following commands were used:

```shell
# Receiver
cargo run --bin receiver

# Sender
cargo run --bin auto_sender -- --duration 10 --rate 100 --target localhost:8001
```

## Cleanup
After running the test cases, reset all network manipulations using:

```shell
# Clean up
sudo dnctl flush
sudo pfctl -f /etc/pf.conf
```