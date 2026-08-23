#!/usr/bin/env python3
# [v3.7.0] Phase 47 Task M-4: Mock MCP 서버.
# MCP JSON-RPC 2.0 프로토콜을 모사하여 E2E 테스트에서 사용.
# stdin으로 JSON-RPC 요청을 받고, stdout으로 응답을 반환.
# 지원 메서드:
#   - initialize: 서버 정보 반환
#   - notifications/initialized: 알림 (무시)
#   - tools/list: mock 도구 목록 반환
#   - tools/call: 도구 호출 결과 반환

import sys
import json
import os
import subprocess
import time

def handle_request(request):
    """JSON-RPC 2.0 요청을 처리하여 응답 딕셔너리를 반환."""
    method = request.get("method", "")
    request_id = request.get("id")

    if method == "initialize":
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {
                    "name": "mock-mcp-server",
                    "version": "1.0.0"
                }
            }
        }
    elif method == "notifications/initialized":
        # 알림이므로 응답하지 않음
        return None
    elif method == "tools/list":
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "tools": [
                    {
                        "name": "get_weather",
                        "description": "현재 날씨 정보를 반환합니다.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "city": {
                                    "type": "string",
                                    "description": "도시 이름"
                                }
                            },
                            "required": ["city"]
                        }
                    },
                    {
                        "name": "read_file",
                        "description": "파일 내용을 읽어 반환합니다.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": {
                                    "type": "string",
                                    "description": "파일 경로"
                                }
                            },
                            "required": ["path"]
                        }
                    },
                    {
                        "name": "read_env",
                        "description": "테스트 환경 변수 조회",
                        "inputSchema": {
                            "type": "object",
                            "properties": {"name": {"type": "string"}},
                            "required": ["name"]
                        }
                    },
                    {
                        "name": "hang",
                        "description": "취소 테스트용 지연 도구",
                        "inputSchema": {"type": "object", "properties": {}}
                    },
                    {
                        "name": "huge_output",
                        "description": "크기 제한 테스트 도구",
                        "inputSchema": {"type": "object", "properties": {}}
                    },
                    {
                        "name": "spawn_child",
                        "description": "process group 종료 테스트 도구",
                        "inputSchema": {"type": "object", "properties": {}}
                    }
                ]
            }
        }
    elif method == "tools/call":
        params = request.get("params", {})
        tool_name = params.get("name", "")
        arguments = params.get("arguments", {})

        if tool_name == "get_weather":
            city = arguments.get("city", "unknown")
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "content": [
                        {"type": "text", "text": f"{city}: 맑음, 22°C"}
                    ]
                }
            }
        elif tool_name == "read_file":
            path = arguments.get("path", "unknown")
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "content": [
                        {"type": "text", "text": f"파일 내용: {path}의 데이터"}
                    ]
                }
            }
        elif tool_name == "error_tool":
            # 에러 케이스: isError=true
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "isError": True,
                    "content": [
                        {"type": "text", "text": "도구 실행 중 오류가 발생했습니다."}
                    ]
                }
            }
        elif tool_name == "read_env":
            name = arguments.get("name", "")
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "content": [
                        {"type": "text", "text": os.environ.get(name, "<unset>")}
                    ]
                }
            }
        elif tool_name == "hang":
            time.sleep(60)
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {"content": [{"type": "text", "text": "late"}]}
            }
        elif tool_name == "huge_output":
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "content": [{"type": "text", "text": "x" * (1024 * 1024 + 1)}]
                }
            }
        elif tool_name == "spawn_child":
            child = subprocess.Popen(
                [sys.executable, "-c", "import time; time.sleep(60)"],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "content": [{"type": "text", "text": str(child.pid)}]
                }
            }
        else:
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {
                    "code": -32601,
                    "message": f"Unknown tool: {tool_name}"
                }
            }
    else:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32601,
                "message": f"Method not found: {method}"
            }
        }

def main():
    """stdin에서 JSON-RPC 요청을 한 줄씩 읽어 처리."""
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
        except json.JSONDecodeError:
            continue

        response = handle_request(request)
        if response is not None:
            sys.stdout.write(json.dumps(response) + "\n")
            sys.stdout.flush()

if __name__ == "__main__":
    main()
