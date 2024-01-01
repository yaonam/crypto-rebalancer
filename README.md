# crypto-rebalancer

## Components

### Broker

Broker handles connections to the exchange. Can create multiple.

### BrokerStatic

BrokerStatic provides the interface for order and balance handling for the exchange. Only one is needed.

### Strategy

Strategy implements the logic for the rebalancing strategy. Can create multiple.

### Signer

Holds the account's private key and signs transactions.