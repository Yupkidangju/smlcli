import re

with open("src/tests/audit_regression.rs", "r") as f:
    content = f.read()

def replace_exec(match):
    return '''ToolCall {
        name: "ExecShell".to_string(),
        args: serde_json::json!({
            "command": ''' + match.group(1) + ''',
            "cwd": ''' + match.group(2) + ''',
            "safe_to_auto_run": ''' + match.group(3) + '''
        }),
    }'''

def replace_write(match):
    return '''ToolCall {
        name: "WriteFile".to_string(),
        args: serde_json::json!({
            "path": ''' + match.group(1) + ''',
            "content": ''' + match.group(2) + ''',
            "overwrite": ''' + match.group(3) + '''
        }),
    }'''

def replace_read(match):
    return '''ToolCall {
        name: "ReadFile".to_string(),
        args: serde_json::json!({
            "path": ''' + match.group(1) + ''',
            "start_line": ''' + match.group(2) + ''',
            "end_line": ''' + match.group(3) + '''
        }),
    }'''

content = re.sub(r'ToolCall::ExecShell\s*\{\s*command:\s*(.*?),\s*cwd:\s*(.*?),\s*safe_to_auto_run:\s*(.*?),\s*\}', replace_exec, content, flags=re.DOTALL)
content = re.sub(r'ToolCall::WriteFile\s*\{\s*path:\s*(.*?),\s*content:\s*(.*?),\s*overwrite:\s*(.*?),\s*\}', replace_write, content, flags=re.DOTALL)
content = re.sub(r'ToolCall::ReadFile\s*\{\s*path:\s*(.*?),\s*start_line:\s*(.*?),\s*end_line:\s*(.*?),\s*\}', replace_read, content, flags=re.DOTALL)

with open("src/tests/audit_regression.rs", "w") as f:
    f.write(content)

print("Done")
