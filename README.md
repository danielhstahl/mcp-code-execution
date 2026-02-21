# An MCP Server for safe local code execution

Yes, there are a million MCP Servers for sandboxed code execution.  But this one is mine.

This is intended to run as a webservice on your local machine or local (trusted) network.  Code is executed in docker containers that do not run as root.

## build image

Make sure to build the docker images before running the mcp:

`docker build -t python-no-root -f docker/python.Dockerfile ./docker`

`docker build -t rust-no-root -f docker/rust.Dockerfile ./docker`

`docker build -t js-no-root -f docker/javascript.Dockerfile ./docker`

Test the images:

`docker run -it -v $(pwd):/app -e TYPE=default -w /app python-no-root ./integration-tests/helloworld.py`

`docker run -it -v $(pwd):/app -w /app rust-no-root cargo run`

`docker run -it -v $(pwd):/app -e TYPE=default -w /app js-no-root ./integration-tests/helloworld.js`

## Accessing MCP

`npx @modelcontextprotocol/inspector http://127.0.0.1:8000/mcp --cli --method tools/list`

`npx @modelcontextprotocol/inspector --cli http://127.0.0.1:8000/mcp --method tools/call --tool-name run_python --tool-arg entry_file=helloworld.py --tool-arg project_dir=$(pwd)`

`npx @modelcontextprotocol/inspector --cli http://127.0.0.1:8000/mcp --method tools/call --tool-name run_rust --tool-arg execution_type=test --tool-arg project_dir=$(pwd)`

## Adding MCP to Claude Code (Ollama)

`ollama launch claude -- mcp add --transport http run_code http://127.0.0.1:8000/mcp`

## Adding to native Claude Code

`claude mcp add --transport http run_code http://127.0.0.1:8000/mcp`
