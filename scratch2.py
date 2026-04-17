import re

with open("src/tests/audit_regression.rs", "r") as f:
    content = f.read()

# Replace None with serde_json::Value::Null within args: serde_json::json!({...})
content = content.replace('"cwd": None,', '"cwd": serde_json::Value::Null,')
content = content.replace('"start_line": None,', '"start_line": serde_json::Value::Null,')
content = content.replace('"end_line": None,', '"end_line": serde_json::Value::Null,')

with open("src/tests/audit_regression.rs", "w") as f:
    f.write(content)

print("Done")
