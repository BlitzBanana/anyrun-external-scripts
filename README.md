# External Script Plugin

An Anyrun plugin that executes custom shell scripts and displays their results.

## Install

```bash
git clone https://github.com/BlitzBanana/anyrun-external-script
cd anyrun-external-script
cargo build --release
cp target/release/libanyrun_external_script.so ~/.config/anyrun/plugins/
```

## Configuration

Create a file named `external-scripts.ron` in your Anyrun config directory:

```ron
Config(
    debug: false,
    shell: "bash",
    scripts: [
        (
            prefix: "cmd",
            command: "get_entries.sh",
            cache: true,
        ),
    ],
)
```

## Script Output Format

Scripts must output valid JSON with the following entry format:

```json
[
  {
    "title": "Display Title",
    "description": "Optional description", // Optional
    "icon": "optional_icon", // Optional
    "command": "command to run when selected", // Optional
    "score": 100 // Optional, if not set will be set using `rust_fuzzy_search`.
  }
]
```

## Configuration Options

- **debug**: Prints debug logs to stdout
- **shell**: The shell to use for script and entry command execution
- **prefix**: The prefix that triggers this script
- **command**: The shell command to execute
- **cache**: Whether to cache results at startup (true/false). Non cached scripts will get the query (unprefixed as argument).

## Fuzzy Matching

Results are automatically matched against your query using fuzzy search. Higher-scoring matches appear first.
