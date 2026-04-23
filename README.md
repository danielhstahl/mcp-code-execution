# An MCP Server for safe local code execution

Yes, there are a million MCP Servers for sandboxed code execution.  But this one is mine.

There are two modes.

* Run the binary directly on your local machine or local (trusted) network.  Code is then executed in docker containers that do not run as root.
* Run the MCP in an isolated docker container that directly runs code.  This is ideal when your entire AI harness is dockerized.

## Run binary directly on local machine with docker images spun up

This option makes the most sense if you are running something like Claude Code locally and want to have sandboxed code execution.

### Build images

Make sure to build the docker images before running the mcp:

`docker build -t python-no-root -f docker/python.Dockerfile ./docker`

`docker build -t rust-no-root -f docker/rust.Dockerfile ./docker`

`docker build -t js-no-root -f docker/javascript.Dockerfile ./docker`

Test the images:

`docker run -it -v $(pwd):/app -e TYPE=default -w /app python-no-root ./integration-tests/helloworld.py`

`docker run -it -v $(pwd):/app -w /app rust-no-root cargo run`

`docker run -it -v $(pwd):/app -e TYPE=default -w /app js-no-root ./integration-tests/helloworld.js`

### Run MCP

`cargo run`

## Run MCP in isolated container

This option makes the most sense if you are running a code compilation harness as part of a larger deployment (eg, with docker compose).

### Build image

```sh
docker build -t main-docker -f docker/Dockerfile ./
```

### Run MCP

```sh
docker run -p 8000:8000 \
-v $(pwd):/app/code_location \
--add-host=host.docker.internal:host-gateway \
main-docker
```

## Accessing MCP

```sh
npx @modelcontextprotocol/inspector http://127.0.0.1:8000/mcp --cli --method tools/list
```

Running Python:

```sh
npx @modelcontextprotocol/inspector \
--cli http://127.0.0.1:8000/mcp \
--method tools/call --tool-name run_python \
--tool-arg entry_file=helloworld.py \
--tool-arg project_dir=integration-tests
```

Running Javascript:

```sh
npx @modelcontextprotocol/inspector \
--cli http://127.0.0.1:8000/mcp \
--method tools/call --tool-name run_javascript \
--tool-arg entry_file=helloworld.js \
--tool-arg project_dir=integration-tests
```

Running Rust:

```sh
npx @modelcontextprotocol/inspector \
--cli http://127.0.0.1:8000/mcp \
--method tools/call --tool-name run_rust \
--tool-arg execution_type=test \
--tool-arg project_dir=.
```

## Adding MCP to Claude Code (Ollama)

`ollama launch claude -- mcp add --transport http run_code http://127.0.0.1:8000/mcp`

## Adding to native Claude Code

`claude mcp add --transport http run_code http://127.0.0.1:8000/mcp`
