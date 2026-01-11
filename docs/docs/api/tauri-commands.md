---
sidebar_position: 1
---

# Tauri Commands API Reference

MINA exposes various Tauri commands for backend functionality. These can be called from the frontend using `invoke()`.

## Automation Commands

### Scripts

#### `list_scripts`
List all available scripts.

```typescript
const scripts = await invoke<Script[]>("list_scripts");
```

#### `get_script`
Get a specific script by ID.

```typescript
const script = await invoke<Script>("get_script", { id: 1 });
```

#### `create_script`
Create a new script.

```typescript
const scriptId = await invoke<number>("create_script", {
  name: "My Script",
  content: "console.log('Hello');",
  language: "javascript",
});
```

#### `update_script`
Update an existing script.

```typescript
await invoke("update_script", {
  id: 1,
  name: "Updated Name",
  content: "// Updated content",
});
```

#### `execute_script`
Execute a script.

```typescript
const result = await invoke<ExecutionResult>("execute_script", {
  id: 1,
  inputs: {},
});
```

### Workflows

#### `list_workflows`
List all workflows.

```typescript
const workflows = await invoke<Workflow[]>("list_workflows");
```

#### `get_workflow`
Get a specific workflow by ID.

```typescript
const workflow = await invoke<Workflow>("get_workflow", { id: 1 });
```

#### `create_workflow`
Create a new workflow.

```typescript
const workflowId = await invoke<number>("create_workflow", {
  name: "My Workflow",
  description: "Description",
  triggerType: "manual",
  triggerConfig: "{}",
  steps: "[]",
});
```

#### `execute_workflow`
Execute a workflow.

```typescript
const executionId = await invoke<number>("execute_workflow", {
  workflowId: 1,
  triggerData: null,
});
```

#### `get_workflow_executions`
Get workflow execution history.

```typescript
const executions = await invoke<WorkflowExecution[]>(
  "get_workflow_executions",
  {
    workflowId: 1,
    limit: 20,
  }
);
```

## Market Data Commands

#### `get_market_price`
Get current market price for a ticker.

```typescript
const price = await invoke<MarketPrice>("get_market_price", {
  ticker: "AAPL",
});
```

#### `get_price_history`
Get historical price data.

```typescript
const history = await invoke<PriceHistory[]>("get_price_history", {
  ticker: "AAPL",
  days: 30,
});
```

#### `list_stock_news`
List stock news items.

```typescript
const news = await invoke<StockNewsItem[]>("list_stock_news", {
  ticker: "AAPL",
  limit: 20,
});
```

## System Commands

#### `get_system_metrics`
Get current system metrics.

```typescript
const metrics = await invoke<SystemMetrics>("get_system_metrics");
```

#### `list_processes`
List running processes.

```typescript
const processes = await invoke<ProcessInfo[]>("list_processes");
```

## WebSocket Commands

#### `ws_connect`
Connect to WebSocket server.

```typescript
const connectionId = await invoke<string>("ws_connect", {
  topics: ["system-metrics", "market-data"],
});
```

#### `ws_subscribe`
Subscribe to topics.

```typescript
await invoke("ws_subscribe", {
  connectionId: "uuid",
  topics: ["stock-news"],
});
```

## Type Definitions

### Script

```typescript
interface Script {
  id: number;
  name: string;
  content: string;
  language: string;
  created_at: number;
  updated_at: number;
  enabled: boolean;
}
```

### Workflow

```typescript
interface Workflow {
  id: number;
  name: string;
  description?: string;
  trigger_type: string;
  trigger_config: string;
  steps: string;
  created_at: number;
  enabled: boolean;
}
```

### MarketPrice

```typescript
interface MarketPrice {
  ticker: string;
  price: number;
  change: number;
  change_percent: number;
  volume: number;
  timestamp: number;
}
```

## Error Handling

All commands may throw errors. Always wrap in try-catch:

```typescript
try {
  const result = await invoke("some_command", {});
} catch (error) {
  console.error("Command failed:", error);
}
```

## WebSocket Events

Listen for real-time updates:

```typescript
import { listen } from "@tauri-apps/api/event";

const unlisten = await listen("ws-message", (event) => {
  const message = event.payload;
  // Handle message
});
```
