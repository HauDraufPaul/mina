---
sidebar_position: 2
---

# WebSocket Events

MINA uses WebSocket events for real-time updates. Listen to these events to receive live data.

## Event Types

### System Events

#### `system-metrics`
System metrics update (sent every second).

```typescript
{
  type: "system-metrics",
  data: {
    cpu: { usage: 45.2, cores: 8, frequency: 2400 },
    memory: { total: 16384, used: 8192, free: 8192, usage: 50.0 },
    disk: { total: 1000000, used: 500000, free: 500000, usage: 50.0 },
    network: { rx: 1000000, tx: 500000, rxSpeed: 1000, txSpeed: 500 }
  },
  timestamp: 1234567890
}
```

#### `process-update`
Process information update.

```typescript
{
  type: "process-update",
  data: {
    pid: 1234,
    name: "chrome",
    cpu_usage: 5.2,
    memory_usage: 1024
  },
  timestamp: 1234567890
}
```

#### `network-update`
Network interface update.

```typescript
{
  type: "network-update",
  data: {
    name: "eth0",
    rx: 1000000,
    tx: 500000
  },
  timestamp: 1234567890
}
```

### Market Events

#### `market-data`
Single market price update.

```typescript
{
  type: "market-data",
  data: {
    ticker: "AAPL",
    price: 150.25,
    change: 2.5,
    change_percent: 1.69,
    volume: 1000000,
    timestamp: 1234567890
  },
  timestamp: 1234567890
}
```

#### `market-data-batch`
Batch market price updates.

```typescript
{
  type: "market-data-batch",
  data: [
    { ticker: "AAPL", price: 150.25, ... },
    { ticker: "GOOGL", price: 2500.00, ... }
  ],
  timestamp: 1234567890
}
```

#### `stock-news`
Single stock news item.

```typescript
{
  type: "stock-news",
  data: {
    id: 1,
    title: "Apple announces new product",
    content: "...",
    ticker: "AAPL",
    published_at: 1234567890
  },
  timestamp: 1234567890
}
```

#### `stock-news-batch`
Batch stock news items.

```typescript
{
  type: "stock-news-batch",
  data: [...],
  timestamp: 1234567890
}
```

### Automation Events

#### `workflow-execution`
Workflow execution update.

```typescript
{
  type: "workflow-execution",
  data: {
    id: 1,
    workflow_id: 1,
    status: "running",
    started_at: 1234567890
  },
  timestamp: 1234567890
}
```

### Temporal Events

#### `temporal-alert`
Temporal alert triggered.

```typescript
{
  type: "temporal-alert",
  data: {
    id: 1,
    rule_id: 1,
    fired_at: 1234567890,
    payload_json: {}
  },
  timestamp: 1234567890
}
```

#### `temporal-job-status`
Temporal job status update.

```typescript
{
  type: "temporal-job-status",
  data: {
    job: "rebuild-events-mvp",
    touched_events: 100,
    days_back: 30
  },
  timestamp: 1234567890
}
```

## Listening to Events

### Using the Realtime Service

```typescript
import { realtimeService } from "@/services/realtimeService";

const unsubscribe = realtimeService.subscribe("market-data", (data) => {
  console.log("Market data:", data);
});
```

### Using Tauri Events Directly

```typescript
import { listen } from "@tauri-apps/api/event";

const unlisten = await listen("ws-message", (event) => {
  const { type, data } = event.payload;
  if (type === "market-data") {
    // Handle market data
  }
});
```

### Using the WebSocket Hook

```typescript
import { useWebSocket } from "@/hooks/useWebSocket";

const { isConnected, subscribe } = useWebSocket({
  topics: ["market-data", "system-metrics"],
  onMessage: (message) => {
    console.log("Received:", message);
  },
});
```

## Event Topics

Subscribe to specific topics when connecting:

- `system-metrics` - System metrics updates
- `market-data` - Market price updates
- `stock-news` - Stock news updates
- `workflow-execution` - Workflow execution updates
- `temporal-alert` - Temporal alerts
- `*` - All events

## Best Practices

- Subscribe only to topics you need
- Unsubscribe when components unmount
- Handle errors gracefully
- Use debouncing for high-frequency events
- Cache event data when appropriate
