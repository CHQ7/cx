# ga-core Python Client

Python client for the ga-core Rust HTTP API.

## Installation

```bash
pip install -e .
```

## Usage

```python
from ga_core_client import GaCoreClient

client = GaCoreClient("http://localhost:8080")

# Check health
print(client.health())  # "OK"

# Run a task
result = client.run(
    system_prompt="You are a helpful assistant",
    user_input="Hello!",
)
print(result)

# Get tool schema
schema = client.get_schema()
print(schema)
```
