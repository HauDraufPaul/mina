---
sidebar_position: 2
---

# Scripts Guide

Learn how to create and use scripts in MINA.

## What are Scripts?

Scripts are executable code snippets written in JavaScript or TypeScript. They can be run independently or as part of workflows.

## Creating Scripts

### Basic Script

```javascript
// Simple script example
console.log("Hello from MINA!");

// Access Tauri commands
const { invoke } = await import("@tauri-apps/api/core");
const metrics = await invoke("get_system_metrics");
console.log("CPU Usage:", metrics.cpu.usage);
```

### Script with Inputs

```javascript
// Script that accepts inputs
const ticker = inputs.ticker || "AAPL";

const { invoke } = await import("@tauri-apps/api/core");
const price = await invoke("get_market_price", { ticker });

return {
  ticker,
  price: price.price,
  timestamp: Date.now()
};
```

### Async Script

```javascript
// Async operations
async function fetchData() {
  const { invoke } = await import("@tauri-apps/api/core");
  
  const [price, news] = await Promise.all([
    invoke("get_market_price", { ticker: "AAPL" }),
    invoke("list_stock_news", { ticker: "AAPL", limit: 5 })
  ]);
  
  return { price, news };
}

const result = await fetchData();
return result;
```

## Available APIs

### Tauri Commands

Access all Tauri commands via `invoke()`:

```javascript
const { invoke } = await import("@tauri-apps/api/core");

// System commands
const metrics = await invoke("get_system_metrics");
const processes = await invoke("list_processes");

// Market data
const price = await invoke("get_market_price", { ticker: "AAPL" });

// Automation
const scripts = await invoke("list_scripts");
```

### File System

```javascript
import { readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import { appDataDir, join } from "@tauri-apps/api/path";

const dataDir = await appDataDir();
const filePath = await join(dataDir, "data.txt");
const content = await readTextFile(filePath);
```

### HTTP Requests

```javascript
const response = await fetch("https://api.example.com/data");
const data = await response.json();
return data;
```

## Error Handling

Always handle errors:

```javascript
try {
  const { invoke } = await import("@tauri-apps/api/core");
  const result = await invoke("some_command", {});
  return result;
} catch (error) {
  console.error("Error:", error);
  throw error; // Re-throw to mark script as failed
}
```

## Best Practices

### Code Organization

- Keep scripts focused and single-purpose
- Use clear, descriptive names
- Add comments for complex logic
- Break complex scripts into functions

### Performance

- Use async/await for I/O operations
- Cache frequently accessed data
- Avoid blocking operations
- Optimize database queries

### Security

- Validate all inputs
- Sanitize user data
- Use parameterized queries
- Check permissions before file operations

### Testing

- Test scripts before adding to workflows
- Use sample data for testing
- Verify error handling
- Check edge cases

## Common Patterns

### Data Processing

```javascript
// Process and transform data
const { invoke } = await import("@tauri-apps/api/core");
const news = await invoke("list_stock_news", { limit: 100 });

const processed = news.map(item => ({
  title: item.title,
  sentiment: analyzeSentiment(item.content),
  timestamp: item.published_at
}));

return processed;
```

### Conditional Logic

```javascript
const { invoke } = await import("@tauri-apps/api/core");
const price = await invoke("get_market_price", { ticker: "AAPL" });

if (price.price > 150) {
  return { alert: "Price is high", price: price.price };
} else {
  return { alert: "Price is normal", price: price.price };
}
```

### Loops and Iteration

```javascript
const { invoke } = await import("@tauri-apps/api/core");
const tickers = ["AAPL", "GOOGL", "MSFT"];

const prices = [];
for (const ticker of tickers) {
  const price = await invoke("get_market_price", { ticker });
  prices.push({ ticker, price: price.price });
}

return prices;
```

## Debugging

### Console Logging

```javascript
console.log("Debug info:", data);
console.error("Error occurred:", error);
```

### Return Values

Return data for inspection:

```javascript
return {
  step1: result1,
  step2: result2,
  final: processed
};
```

### Error Messages

Provide clear error messages:

```javascript
if (!inputs.ticker) {
  throw new Error("Ticker is required");
}
```

## Integration with Workflows

Scripts can be used in workflows:

1. Create the script
2. Add "ExecuteScript" step to workflow
3. Configure script ID and inputs
4. Use output variable in subsequent steps

## Examples

See the [Automation Circuit documentation](../modules/automation-circuit) for more examples and use cases.
