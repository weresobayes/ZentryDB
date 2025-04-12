# Command Design Guide

## Protocol Buffers

```protobuf
syntax = "proto3";
package zentry;

message Account {
    string id = 1;
    string name = 2;
    string account_type = 3;
    string created_at = 4;  // RFC3339 timestamp
    string system_id = 5;   // Reference to the currency system
}

message CreateAccountRequest {
    string name = 1;
    string account_type = 2;
    string system_id = 3;   // Required: currency system for the account
}

message CreateAccountResponse {
    Account account = 1;
}

message ListAccountsRequest {}

message ListAccountsResponse {
    repeated Account accounts = 1;
}

message GetAccountRequest {
    string id = 1;
}

message GetAccountResponse {
    Account account = 1;
}

message System {
    string id = 1;          // Currency code (e.g. "USD", "IDR")
    string description = 2;  // Human readable description
}

message CreateSystemRequest {
    string id = 1;          // Currency code
    string description = 2;  // Optional description
}

message CreateSystemResponse {
    System system = 1;
}

message ListSystemsRequest {}

message ListSystemsResponse {
    repeated System systems = 1;
}

message GetSystemRequest {
    string id = 1;
}

message GetSystemResponse {
    System system = 1;
}

service AccountService {
    rpc Create(CreateAccountRequest) returns (CreateAccountResponse);
    rpc List(ListAccountsRequest) returns (ListAccountsResponse);
    rpc Get(GetAccountRequest) returns (GetAccountResponse);
}

service SystemService {
    rpc Create(CreateSystemRequest) returns (CreateSystemResponse);
    rpc List(ListSystemsRequest) returns (ListSystemsResponse);
    rpc Get(GetSystemRequest) returns (GetSystemResponse);
}
```

## JSON Format Examples

### Create

```shell
$ zentry account create savings asset --system USD
{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "savings",
    "account_type": "asset",
    "system_id": "USD",
    "created_at": "2025-04-13T06:53:06+08:00"
}
```

### List

```shell
$ zentry account list
[
    {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "savings",
        "account_type": "asset",
        "system_id": "USD",
        "created_at": "2025-04-13T06:53:06+08:00"
    },
    {
        "id": "7f8e9d2c-b3a1-4f5e-9c8b-2d6a5e4f3b2a",
        "name": "credit_card",
        "account_type": "liability",
        "system_id": "IDR",
        "created_at": "2025-04-13T06:53:15+08:00"
    }
]
```

### Get

```shell
$ zentry account get 550e8400-e29b-41d4-a716-446655440000
{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "savings",
    "account_type": "asset",
    "system_id": "USD",
    "created_at": "2025-04-13T06:53:06+08:00"
}
```

## System

### Create

```shell
$ zentry system create USD "United States Dollar"
{
    "id": "USD",
    "description": "United States Dollar"
}
```

### List

```shell
$ zentry system list
[
    {
        "id": "USD",
        "description": "United States Dollar"
    },
    {
        "id": "IDR",
        "description": "Indonesian Rupiah"
    }
]
```

### Get

```shell
$ zentry system get USD
{
    "id": "USD",
    "description": "United States Dollar"
}
```

## Conversion Graph

### Set

```shell
$ zentry conversion_graph set USD -> IDR 14000.0
{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "graph": "USD -> IDR",
    "rate": 14000.0,
    "rate_since": "2025-04-13T06:27:15+08:00"
}
```

### List

```shell
$ zentry conversion_graph list
[
    {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "graph": "USD -> IDR",
        "rate": 14000.0,
        "rate_since": "2025-04-13T06:27:15+08:00"
    },
    {
        "id": "7f8e9d2c-b3a1-4f5e-9c8b-2d6a5e4f3b2a",
        "graph": "IDR -> USD",
        "rate": 7.142857142857143,
        "rate_since": "2025-04-13T06:28:30+08:00"
    }
]
```

### Get

```shell
$ zentry conversion_graph get USD -> IDR
{
    "graph": "USD -> IDR",
    "rate": 14000.0,
    "rate_since": "2025-04-13T06:27:15+08:00"
}
```

## Next milestone:

- Add transaction
- Add entry
- Add verbose option to conversion_graph command
    > for example: zentry conversion_graph get USD - IDR --verbose  
    > the command response will describe the relation between two systems  
