---
sidebar_position: 1
---

# Automation Circuit

Automation Circuit is MINA's workflow and script automation system. It allows you to create reusable scripts and complex workflows to automate tasks across your system.

## Overview

Automation Circuit provides two main features:

1. **Scripts**: Executable code snippets written in JavaScript or TypeScript
2. **Workflows**: Sequences of steps that automate complex processes

## Scripts

Scripts are executable code snippets that can be run on demand or as part of workflows.

### Creating a Script

1. Navigate to **Automation Circuit**
2. Click the **Scripts** tab
3. Click **New Script**
4. Enter a name and select a language (JavaScript or TypeScript)
5. Write your script code
6. Click **Save** to store the script

### Executing Scripts

Scripts can be executed in several ways:

- **Manually**: Click the play button next to a script in the Scripts tab
- **As part of a workflow**: Use the "ExecuteScript" step type
- **Via command bar**: Use the command bar to execute scripts by name

### Script Environment

Scripts have access to:

- System information via Tauri commands
- Other scripts and workflows
- File system (with appropriate permissions)
- Network requests
- Database access

## Workflows

Workflows are sequences of steps that automate complex multi-step processes.

### Creating a Workflow

1. Click the **Workflows** tab
2. Click **New Workflow**
3. Configure the workflow:
   - **Name**: Descriptive name for the workflow
   - **Description**: Optional description
   - **Trigger Type**: How the workflow is triggered
     - **Manual**: Run on demand
     - **Scheduled**: Run on a schedule (cron format)
     - **Event-based**: Triggered by events
   - **Trigger Config**: JSON configuration for the trigger
4. Add steps using the workflow editor
5. Save and enable the workflow

### Step Types

Workflows support several step types:

#### ExecuteScript
Run a script with optional inputs and capture output.

```json
{
  "type": "ExecuteScript",
  "script_id": 1,
  "inputs": {
    "param1": "value1"
  },
  "output_var": "result"
}
```

#### CallCommand
Execute a system command or Tauri command.

```json
{
  "type": "CallCommand",
  "command": "get_market_price",
  "args": {
    "ticker": "AAPL"
  },
  "output_var": "price"
}
```

#### Condition
Branch based on a condition.

```json
{
  "type": "Condition",
  "condition": "price > 100",
  "if_true": [...],
  "if_false": [...]
}
```

#### Wait
Pause execution for a specified duration.

```json
{
  "type": "Wait",
  "duration_seconds": 5
}
```

#### SendAlert
Send a notification or alert.

```json
{
  "type": "SendAlert",
  "message": "Workflow completed",
  "severity": "info",
  "channel": "desktop"
}
```

#### SetVariable
Store a value for later use in the workflow.

```json
{
  "type": "SetVariable",
  "name": "total",
  "value": 100
}
```

#### Loop
Iterate over a collection of items.

```json
{
  "type": "Loop",
  "items": [1, 2, 3],
  "item_var": "item",
  "steps": [...]
}
```

### Workflow Editor

The workflow editor provides a visual interface for building workflows:

- **Add Step**: Click the "+" button to add a new step
- **Edit Step**: Click on a step to edit its configuration
- **Reorder Steps**: Drag and drop to reorder
- **Delete Step**: Click the trash icon to remove a step

### Workflow Execution

Workflows can be executed:

- **Manually**: Click the play button in the Workflows tab
- **Scheduled**: Automatically based on the schedule configuration
- **Event-based**: Triggered by system or application events

### Monitoring Executions

View workflow executions in the **Executions** tab:

- See execution status (running, completed, failed)
- View execution logs and output
- Debug failed executions
- View execution history

## Best Practices

### Scripts

- Keep scripts focused and reusable
- Use clear, descriptive names
- Add comments for complex logic
- Handle errors gracefully
- Test scripts before adding to workflows

### Workflows

- Start simple and add complexity gradually
- Use variables to pass data between steps
- Add error handling steps
- Test workflows with sample data
- Monitor execution logs for debugging
- Document workflow purpose and configuration

### Performance

- Avoid long-running scripts in workflows
- Use async operations when possible
- Cache frequently accessed data
- Optimize database queries
- Monitor resource usage

## Examples

### Example: Daily System Check

A workflow that checks system health daily:

1. **ExecuteScript**: Run system health check script
2. **Condition**: If CPU usage > 80%
3. **SendAlert**: Send warning notification
4. **Wait**: Wait 5 minutes
5. **ExecuteScript**: Run cleanup script

### Example: Market Alert

A workflow that monitors stock prices:

1. **CallCommand**: Get current price for AAPL
2. **SetVariable**: Store price
3. **Condition**: If price > threshold
4. **SendAlert**: Send price alert
5. **CallCommand**: Log alert to database

## Troubleshooting

### Script Execution Fails

- Check script syntax
- Verify required permissions
- Check error logs in the Executions tab
- Test script manually first

### Workflow Not Triggering

- Verify workflow is enabled
- Check trigger configuration
- Review schedule format (if scheduled)
- Check event subscriptions (if event-based)

### Performance Issues

- Review script execution times
- Check for infinite loops
- Monitor system resources
- Optimize database queries

## API Reference

See the [API Reference](../api/tauri-commands) for available Tauri commands that can be used in scripts and workflows.
