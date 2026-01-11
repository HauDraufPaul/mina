---
sidebar_position: 1
---

# Workflow Guide

Learn how to create and manage workflows in MINA.

## What are Workflows?

Workflows are sequences of steps that automate complex processes. They can include scripts, conditions, loops, and more.

## Creating Your First Workflow

1. Navigate to **Automation Circuit**
2. Click the **Workflows** tab
3. Click **New Workflow**
4. Enter a name and description
5. Select a trigger type
6. Add steps using the workflow editor
7. Save and enable

## Workflow Triggers

### Manual Trigger

Run the workflow on demand by clicking the play button.

### Scheduled Trigger

Run the workflow on a schedule using cron syntax:

```
0 9 * * *  # Every day at 9 AM
*/5 * * * *  # Every 5 minutes
0 0 * * 0  # Every Sunday at midnight
```

### Event-based Trigger

Trigger the workflow when specific events occur:

- System events
- Market events
- Custom events

## Step Types Explained

### ExecuteScript

Run a script with inputs and capture output.

**Configuration:**
- Script ID
- Input variables
- Output variable name

### CallCommand

Execute a Tauri command or system command.

**Configuration:**
- Command name
- Arguments
- Output variable name

### Condition

Branch workflow execution based on a condition.

**Configuration:**
- Condition expression
- Steps to execute if true
- Steps to execute if false

### Wait

Pause workflow execution.

**Configuration:**
- Duration in seconds

### SendAlert

Send a notification.

**Configuration:**
- Message
- Severity (info, warning, error)
- Channel (desktop, email, etc.)

### SetVariable

Store a value for later use.

**Configuration:**
- Variable name
- Value

### Loop

Iterate over a collection.

**Configuration:**
- Items to iterate
- Item variable name
- Steps to execute for each item

## Advanced Patterns

### Error Handling

Add error handling to workflows:

1. Wrap risky steps in conditions
2. Check for errors after execution
3. Send alerts on failure
4. Log errors for debugging

### Data Flow

Pass data between steps:

1. Use SetVariable to store values
2. Reference variables in subsequent steps
3. Use output variables from script/command steps
4. Access variables in conditions

### Parallel Execution

While workflows run sequentially, you can:

1. Create multiple workflows for parallel tasks
2. Use async operations in scripts
3. Chain workflows together

## Debugging Workflows

### View Execution Logs

1. Go to the **Executions** tab
2. Click on an execution
3. View step-by-step logs
4. Check for errors

### Test Individual Steps

1. Test scripts separately
2. Verify command outputs
3. Check variable values
4. Validate conditions

### Common Issues

- **Workflow not running**: Check if enabled
- **Steps failing**: Review step configuration
- **Variables not set**: Check variable names
- **Condition not working**: Verify expression syntax

## Best Practices

- Start simple and add complexity
- Document workflow purpose
- Use descriptive variable names
- Add error handling
- Test thoroughly
- Monitor execution logs
- Keep workflows focused
